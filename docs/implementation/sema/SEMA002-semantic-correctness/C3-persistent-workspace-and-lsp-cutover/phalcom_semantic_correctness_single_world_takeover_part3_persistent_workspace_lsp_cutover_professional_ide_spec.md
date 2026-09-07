# Phalcom Semantic Correctness / Single-World Takeover — Part 3 of 3
# Persistent Workspace Lifecycle, Final LSP Cutover, and Professional IDE Presentation

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this specification task-by-task. Every task is independently reviewable and test-gated. Do not collapse the migration, deletion, UX, native-contract repair, and final acceptance work into one commit.
>
> **Implementation order:** Part 1 — Formal Semantic Epistemic Foundation, including its corrections/amendments, and Part 2 — Canonical Semantic Identity, Projection, and Advisory Evidence Takeover are hard prerequisites. Part 3 is the final ownership/lifecycle/consumer cutover. Do not compensate for an incomplete Part 1 or Part 2 by recreating semantic logic in `phalcom-lsp`.

**Goal:** Complete the single-world semantic takeover so that one compiler-owned persistent workspace/module/source lifecycle feeds one `phalcom-semantic::SemanticWorkspaceSession`, which publishes one immutable semantic snapshot consumed directly by all LSP features. At the same time, make the IDE presentation practical: ordinary types look like ordinary types regardless of whether their backing evidence is formal or advisory; provenance and uncertainty are exposed contextually in hover/tooltips rather than encoded as mathematical-looking type syntax. Correct canonical native contracts must participate in formal return proving before any advisory fallback is considered.

**Architecture:** `phalcom-modules` remains the sole authority for project identity, source identity, module resolution, linking, imports, exposure, and source overlays. A new persistent module-workspace lifecycle in `phalcom-modules` retains project and source identity across ordinary edits. `phalcom-semantic::SemanticWorkspaceSession` remains the sole formal/advisory semantic owner and consumes those persistent module products incrementally. `phalcom-semantic::SemanticSnapshot` is the sole published semantic world. `phalcom-lsp` owns protocol adaptation, open-document buffering, request syntax recovery, scheduling/debouncing, rendering, and client notifications—never semantic identity, semantic inference, dispatch, invalidation, or a second formal/advisory cache.

**Repository baseline inspected for this specification:** `aureat/phalcom-lang` `main` at commit `c36586619b1bf8f93429377b31425888b77f7df1` (`feat(semantic): establish epistemic correctness foundations`, 2026-08-24). This is newer than the Part 2 archaeology baseline and already contains the first implementation slice of Part 1, including `EvidenceStatus::{Established, Assumed}` and `EvidenceOrigin`, while the LSP still retains its transitional dual-world publication path.

**Normative predecessor documents:**

- `phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md`
- `phalcom_semantic_correctness_part1_corrections_and_amendments.md`
- `phalcom_semantic_correctness_single_world_takeover_part2_canonical_identity_projection_advisory_takeover_spec.md`

This Part 3 specification **supersedes Part 2 presentation examples wherever Part 2 proposed a visible advisory marker such as `≈ T`**. The semantic separation remains mandatory. The marker does not.

---

# 1. Part 3 scope

Part 3 implements the work previously grouped as SC-14 through SC-19.

## 1.1 SC-14 — Persistent project/module/source lifecycle

Production LSP updates must no longer reconstruct project/module/link state from an ad hoc source catalog on every semantic refresh. Project identity, synthetic standalone identity, canonical module identity, source overlays, and linked module products must have explicit lifetimes and incremental update semantics.

## 1.2 SC-15 — One persistent compiler semantic session

The LSP worker must own or invoke exactly one long-lived compiler `SemanticWorkspaceSession` for the workspace epoch. Production `run_static_workspace_analysis(...)`, `StaticWorkspaceIdentity`, nested `static_snapshot` publication, and the LSP semantic engine/database wrapper disappear.

## 1.3 SC-16 — Every semantic LSP feature consumes one immutable compiler snapshot

Diagnostics, hover, completion, signature help, definition, references, workspace symbols, import/module completion, inlay hints, semantic token refinement, source navigation, and related semantic queries must consume the same pinned `Arc<phalcom_semantic::SemanticSnapshot>` for one request. No feature re-analyzes source to rediscover semantic truth.

## 1.4 SC-17 — Professional IDE presentation and fixed-return correctness

Evidence status is a semantic property, not a user-facing type syntax. Ordinary LSP type labels use canonical Phalcom type spelling. Hover/tooltips explain evidence only when it adds useful context. Trusted native and other canonical callable return contracts must be promoted into formal call-result knowledge before advisory fallback. `System.print` returning formal `Unit` is the concrete release regression.

## 1.5 SC-18 — Physical deletion of the second semantic system

After parity and cutover, remove the remaining semantic implementation under `phalcom-lsp`: duplicate semantic DB/engine, IDs, scopes, occurrence graph, dispatch, advisory inference, summaries, module graph, invalidation and `WorkspaceIndex` semantic authority. Syntax recovery and protocol rendering may remain.

## 1.6 SC-19 — Final parity, lifecycle, UX, and performance release gate

Cold and incremental semantic results must converge. Open/change/close/delete/rename/project-reload lifecycle must preserve canonical identity rules. Stale work must not publish. Semantic feature output must be snapshot-coherent. Structural counters must prove that ordinary edits do not rebuild the world. IDE golden tests must enforce the practical presentation policy.

---

# 2. Hard predecessor contract

Part 3 assumes Parts 1 and 2 have landed in full.

The following are not Part 3 migration conveniences; they are invariants:

```text
formal TypeKnowledge:
    Known(Established | Assumed)
    Unknown(reason)
    Dynamic(reason)

analysis status:
    independent from type knowledge

causal invalidity:
    independent from both

binding:
    persistent contract
    current value knowledge
    consistency
    causal invalidity

source identity:
    canonical declaration/module IDs
    snapshot-scoped local/source-site IDs

source semantic index:
    compiler-owned
    exact occurrences
    reverse target index
    formal attachments
    advisory attachments

advisory:
    compiler-owned ValueShape abstract domain
    separate from TypeKnowledge
    cannot reject code
    cannot upgrade formal Unknown/Assumed/Invalid
    cannot participate in proof

snapshot:
    one immutable compiler publication containing
    formal + source + advisory + module products
```

Part 3 must not solve lifecycle problems by weakening these boundaries.

---

# 3. New normative IDE presentation decision

The compiler needs a rich epistemic model. The editor does not need to look like a theorem prover.

The following distinction is mandatory:

```text
semantic representation:
    exact, explicit, epistemically honest

ordinary IDE surface:
    concise, conventional, code-oriented

hover/tooltips:
    contextual explanation when useful
```

## 3.1 No advisory pseudo-type syntax

The following ordinary UI is forbidden:

```text
≈ String
≈ Option
Observed type: ≈ String
Observed return: ≈ Option
Confidence: flow
```

The visible type is simply:

```text
String
Option
```

The fact that one answer is formal and another is advisory remains available in the backing presentation object and may appear in hover/tooltips when useful.

`≈` must have zero production uses under `phalcom-lsp/src` after Part 3.

## 3.2 Inlay hints look like language syntax

For:

```phalcom
let user = loadUser()
```

a type hint should visually read as though the developer had written:

```phalcom
let user: User = loadUser()
```

The LSP label is:

```text
: User
```

not:

```text
≈ User
```

For an inferred callable return:

```phalcom
load() {
    fetch()
}
```

the return hint is:

```text
 -> Result<Data, Error>
```

not:

```text
 ≈ Result<Data, Error>
```

Formal and advisory knowledge use the same visible type label. Evidence belongs in the tooltip.

## 3.3 Signature help looks like a callable signature

Display:

```text
map(transform: Function) -> List<User>
```

not:

```text
map(transform: ≈ Function) -> ≈ List<User>
```

A parameter/result coming from advisory evidence is not given mathematical decoration.

## 3.4 Completion does not spam epistemic status

Member completion should show normal member names/signatures. If a receiver was inferred through advisory flow, do not append “inferred”, “approximate”, or a glyph to every completion item. That produces noise without helping the developer choose an item.

If the resulting target is later hovered, the hover may explain why that member set was available.

## 3.5 Evidence is contextual information, not boilerplate

Do not render repetitive prose such as:

```text
These types are inferred by Phalcom.
This value was inferred by Phalcom.
The following result was inferred by the semantic analyzer.
```

The fact that the language server produced the hover is obvious.

Prefer information that changes the developer's understanding:

```text
Inferred from `CellNum.new()`.
Declared as `Number`.
Narrowed to `Admin` in this branch.
Return type from native signature.
Specialized here as `Result<User, Error>`.
Declared type; no independent type evidence is available here.
Type unavailable because `T` could not be inferred.
```

Even those lines should be omitted when they merely restate the primary type.

---

# 4. Hover information architecture

Hover is the correct place to make Phalcom's semantic intelligence visible without turning every type label into formal notation.

Each semantic hover is assembled from up to four layers.

## 4.1 Layer 1 — Primary code fact

Always lead with the thing the programmer most likely asked about:

```text
x: CellNum
```

or:

```text
System.print(_ value: Object) -> Unit
```

or:

```text
class CellNum
```

or:

```text
List<User>
```

This line uses ordinary canonical Phalcom spelling.

## 4.2 Layer 2 — Documentation and contract

For declarations, Phaldoc remains high-value content and should appear immediately after the signature/identity.

Example:

```text
System.print(_ value: Object) -> Unit

Writes `value` to standard output followed by a newline.
```

For a binding whose current type is more precise than its declaration:

```text
x: CellNum

Declared as `Number`.
```

This is useful because it explains why completion can offer `CellNum` members despite a broader declaration.

Do not show:

```text
x: CellNum

Declared as `CellNum`.
```

when both are identical.

## 4.3 Layer 3 — Contextual derivation/evidence

Only add an evidence line when one of the following is true:

- the current type differs materially from the written contract;
- the type is flow-narrowed;
- the result is a generic specialization;
- knowledge is only Assumed;
- the answer is advisory-only;
- a native/intrinsic/constructor rule explains a surprising result;
- formal and advisory channels disagree;
- the expression is invalid but still has usable type knowledge;
- the type is Unknown/Dynamic and the reason helps the developer act.

Examples:

```text
Inferred from `CellNum.new()`.
```

```text
Narrowed from `User` after `user is Admin`.
```

```text
Specialized `T` as `String` from argument 1.
```

```text
Return type from native signature.
```

```text
Declared type; no independent value evidence is available here.
```

```text
Inferred from call sites.
```

Avoid implementation vocabulary:

```text
EvidenceStatus::Established
EvidenceOrigin::NativeSignature
Confidence::Interprocedural
RelationOutcome::Proven
```

These are compiler data, not user prose.

## 4.4 Layer 4 — Relevant problem context

When the hovered site is invalid but its type is still known, keep the type and add the smallest useful explanation.

Example:

```phalcom
let x: Int = CellNum.new()
```

Hover on `x`:

```text
x: CellNum

Declared as `Int`.

`CellNum` is not assignable to `Int`.
```

Do not replace `CellNum` with `Unknown`, and do not paste the entire diagnostic stack into the hover.

For:

```phalcom
service.fetch("wrong")
```

if the call target and return are established:

```text
fetch(_ id: Int) -> User

Argument 1 expects `Int`; this call passes `String`.
```

The return may remain `User` even though the call is invalid.

---

# 5. Public wording map for semantic evidence

The LSP renderer should centralize the translation from compiler semantics to developer-facing prose.

The following is a normative starting map.

| Compiler fact | Ordinary label | Hover evidence when useful |
|---|---|---|
| Established + Syntax | normal type | usually omit |
| Established + ConstructorSemantics | normal type | `Inferred from \`Type.selector(...)\`.` |
| Established + CallableSignature | normal type | `Return type from callable signature.` only if non-obvious |
| Established + NativeSignature | normal type | `Return type from native signature.` |
| Established + GenericInference | specialized type | `Specialized here from call arguments.` |
| Established + Flow | narrowed/joined type | `Narrowed by local control flow.` if narrower than declared/base |
| Established + PatternDecomposition | normal type | `Inferred from the matched pattern.` if useful |
| Assumed + DeveloperAnnotation | normal type | `Declared type; no independent value evidence is available here.` |
| Assumed + CallableSignature | normal type | `Using the callable's declared contract.` if useful |
| Advisory exact/flow | normal type | `Inferred from local flow.` when explanation is requested/useful |
| Advisory interprocedural | normal type | `Inferred from call sites.` |
| Advisory heuristic | normal type if UI policy allows | `Best-effort inference from usage.` |
| Unknown underconstrained generic | no fake type | `Type unavailable: a generic argument could not be inferred.` |
| Unknown blocked | no fake type | explain the actionable blocker if one exists |
| Dynamic explicit escape | `Dynamic` | `Dynamic by explicit annotation.` |
| Dynamic reflection/rest boundary | `Dynamic` | concise reason if useful |

This table controls wording only. It must not alter semantic precedence.

---

# 6. Centralize LSP semantic rendering

Create:

```text
phalcom-lsp/src/presentation.rs
```

This module is a pure renderer over compiler-published facts.

It must not:

- query mutable semantic state;
- invoke dispatch;
- resolve names;
- infer a type;
- merge formal/advisory semantics;
- manufacture semantic targets;
- parse source for semantic meaning.

A conceptual API:

```rust
pub struct HoverContext<'a> {
    pub site: &'a phalcom_semantic::SemanticSiteView,
    pub docs: Option<&'a PhaldocDoc>,
    pub source_text: &'a str,
}

pub fn render_hover(context: HoverContext<'_>) -> Option<MarkupContent>;

pub fn render_inlay_type(
    view: &phalcom_semantic::SemanticSiteView,
    kind: InlaySiteKind,
) -> Option<RenderedInlay>;

pub fn render_signature(
    view: &phalcom_semantic::CallablePresentationView,
) -> RenderedSignature;

pub fn render_completion_detail(
    view: &phalcom_semantic::CallablePresentationView,
) -> Option<String>;
```

The exact compiler view names may follow Part 2's final API, but the ownership rule is fixed:

```text
compiler decides:
    what the fact is
    which source site it belongs to
    whether it is formal/advisory
    status/origin/provenance
    canonical type/shape
    exact target

LSP decides:
    how much of that information to show
    wording
    markdown layout
    LSP label shape
```

---

# 7. Formal-first display selection without formal-looking UI

Part 2's semantic precedence remains intact even though the visible syntax is unified.

The renderer must select a primary type in this order:

1. usable formal `Known(T)`;
2. if no usable formal known fact exists, usable advisory shape;
3. explicit `Dynamic` when that is the formal semantic answer;
4. otherwise no concrete type label.

This does **not** mean advisory overrides formal Unknown in the compiler. It means the UI can display useful non-authoritative evidence when the formal channel has no concrete type.

The presentation object must retain both lanes:

```rust
pub struct SemanticTypePresentation {
    pub formal: Option<FormalFactView>,
    pub advisory: Option<AdvisoryFactView>,
}
```

The LSP may choose the advisory lane as the visible primary fallback without converting it into `TypeKnowledge`.

If formal is non-ready because of cancellation/budget/internal failure, do not disguise that infrastructure state as a formally proven advisory answer. A normal advisory type may still be shown if available, but hover should be able to explain that formal analysis is incomplete.

---

# 8. `System.print` is a canonical-contract bug, not merely a presentation bug

Current `main` at `c365866…` contains a three-way inconsistency.

`phalcom-core/src/primitive/system.rs` declares:

```rust
#[phalcom_native_macros::primitive(
    System,
    "print(_)",
    params = [Object],
    returns = Object,
    types = "(Object) -> Object",
    side = class
)]
```

but the Rust implementation ends with:

```rust
Ok(vm.none_value())
```

Meanwhile `docs/spec/current/system.md` currently says `print(_)` returns its argument.

The intended contract ratified for this Part 3 is:

```text
System.print(_): Unit
```

Therefore Part 3 explicitly supersedes the stale `System.print` return text in `docs/spec/current/system.md`.

## 8.1 Required implementation

Change `phalcom-core/src/primitive/system.rs` to:

```rust
#[phalcom_native_macros::primitive(
    System,
    "print(_)",
    params = [Object],
    returns = Unit,
    types = "(Object) -> Unit",
    side = class
)]
pub fn system_class_print(
    vm: &mut VM,
    _receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    for arg in args {
        let text = arg.to_display_string(vm)?;
        print!("{text}");
    }
    println!();
    Ok(vm.unit_value())
}
```

Update the language specification so `System.print(_)` is documented as returning `Unit`, not its argument.

## 8.2 Why this is formally sufficient

`phalcom-semantic/src/types/native.rs` already normalizes canonical native type metadata into:

```text
EvidenceStatus::Established
EvidenceOrigin::NativeSignature
```

and the native registration path lowers `ReturnFlowSpec` / return metadata into canonical callable signatures.

Therefore the desired chain is:

```text
primitive metadata
    returns = Unit
        ↓
native surface catalog
        ↓
register_native_surfaces
        ↓
CallableSemanticSignature return TypeTerm::Canonical(Unit)
        ↓
dispatch resolves System.print(_)
        ↓
call expression
    Established(Unit)
    evidence includes native signature
        ↓
tail expression of caller
        ↓
normal return summary
        ↓
caller return
    Established(Unit)
```

No advisory lane is needed to establish this result.

---

# 9. `System.print` acceptance behavior

Given:

```phalcom
class Demo {
  run() {
    System.print("hello")
  }
}
```

after Parts 1–3:

```text
System.print("hello")
    exact target: System.print(_)
    formal type: Established(Unit)
    origin/support: native signature

Demo.run()
    normal tail return: Unit
    callable inferred return: Established(Unit)
```

IDE:

```text
run() -> Unit
```

Return inlay:

```text
 -> Unit
```

Hover on `System.print`:

```text
System.print(_ value: Object) -> Unit

Writes `value` to standard output followed by a newline.

Return type from native signature.
```

There must be no:

```text
≈ Option
≈ Unit
Observed return
Confidence: interprocedural
```

in this path.

---

# 10. Generalize the `System.print` fix: trusted fixed-return fulfillment

The regression is broader than one primitive.

Whenever dispatch resolves a callable with a canonical fixed return contract, the formal call-result path must be attempted before advisory return summaries.

Applicable sources include:

- canonical native primitive signatures;
- source callables with usable declared return contracts;
- exact constructor semantics;
- intrinsic fixed-return rules;
- generated canonical members whose return is part of their semantic signature;
- non-generic callables with canonical `TypeTerm::Canonical(T)`;
- generic callables whose return can be fully specialized by the formal generic solver.

The precedence is:

```text
1. resolve exact callable identity
2. read canonical callable semantic signature
3. formally instantiate/specialize return if required
4. produce formal call-result knowledge
5. only if formal result cannot be established/assumed under Part 1 rules,
   consult advisory return evidence for IDE enrichment
```

Forbidden:

```text
advisory return Option
    ↓
publish method return Option
    ↓
later notice native signature says Unit
```

The formal callable contract cannot be downstream of advisory inference.

---

# 11. Native return-contract audit

`System.print` demonstrates that semantic import correctness depends on native metadata correctness. Part 3 therefore adds a release-blocking native contract audit.

## 11.1 Audit dimensions

For every native primitive record, inspect:

```text
selector
params
returns
types
ReturnFlowSpec
implementation return behavior
native/source documentation contract
```

The audit asks:

1. Do `returns` and the result of `types = "(...) -> R"` agree?
2. If `flow = receiver`, is the declared result compatible with receiver semantics?
3. If `flow = argument(i)`, does parameter `i` exist and agree with the return representation?
4. If `flow = never`, does the implementation actually never produce a normal value?
5. If a primitive always returns `vm.unit_value()`, is its semantic result `Unit` unless the language specification intentionally defines another wrapper?
6. If it returns `vm.none_value()`, is its contract intentionally `None`/appropriate absence semantics rather than accidentally broad `Object`?
7. Do source/core documentation and native metadata agree?

## 11.2 Do not infer semantic contracts from Rust return mechanics alone

A runtime `Value` representation is not a language type.

For example:

```rust
Ok(vm.none_value())
```

does not automatically mean every such primitive should be `Unit`. `None` and `Unit` are distinct language semantics.

The audit uses the language-level contract as the deciding semantic source. Part 3 explicitly decides only the disputed `System.print` contract: `Unit`.

Other mismatches found by the audit must be reconciled against the current normative language/core specification and recorded in focused tests. A broad `Object` metadata entry must not survive merely because the Rust ABI returns `Value`.

## 11.3 Confirmed adjacent correction: `System.gc`

Current `System.gc` is declared `() -> Object`, returns `vm.none_value()`, while `docs/spec/current/system.md` describes it as returning `None`. `UniverseKey::None` is a canonical universe type, so this mismatch does not need to remain unresolved.

Part 3 therefore also requires:

```rust
#[phalcom_native_macros::primitive(
    System,
    "gc",
    params = [],
    returns = None,
    types = "() -> None",
    side = class
)]
```

The Rust body continues to return `vm.none_value()`.

This correction demonstrates why the audit must not equate `None` with `Unit`:

```text
System.print(_) -> Unit
System.gc        -> None
```

Both are definite trusted native returns and both must be formally established, but they denote different language values/types.

---

# 12. Add native contract coherence tests

Add focused tests under the existing native/core test surface, for example:

```text
phalcom-core/tests/native_contracts.rs
phalcom-semantic/tests/native_contracts.rs
```

Required tests include:

```text
system_print_runtime_returns_unit
system_print_native_metadata_returns_unit
system_print_callable_signature_is_established_unit
system_print_tail_expression_proves_caller_unit
system_print_inlay_is_plain_unit
system_print_hover_has_plain_unit
```

Add a table-driven catalog audit for metadata self-consistency where mechanically checkable:

```rust
for record in NATIVE_SURFACES {
    assert_callable_type_result_matches_returns(record);
    assert_flow_reference_is_valid(record);
}
```

Runtime behavior tests should cover the small set of canonical fixed-result primitives needed to protect against semantic drift. Do not attempt to prove arbitrary Rust bodies by introspection.

---

# 13. Current production lifecycle defect

At current `main`, `phalcom-lsp/src/analysis_service.rs` already embeds a `phalcom_semantic::SemanticWorkspaceSession` in `StaticWorkspaceIdentity`.

That is progress, but production still routes through:

```rust
fn run_static_workspace_analysis(
    source_catalog: &BTreeMap<Url, (FileRevision, Arc<str>, Program)>,
    documents: DocumentModuleMap,
    generation: u64,
    identity: &mut StaticWorkspaceIdentity,
) -> Option<StaticWorkspacePublication>
```

This function reconstructs project/module/link products from an LSP-owned source catalog, invokes `identity.session.update(...)`, then nests the compiler snapshot back inside the LSP semantic engine through `set_static_analysis(...)`.

The ownership graph is still:

```text
LSP source catalog
    ↓
LSP project reconstruction
    ↓
compiler SemanticWorkspaceSession
    ↓
compiler SemanticSnapshot
    ↓
LSP SemanticEngine
    ↓
LSP SemanticSnapshot.static_snapshot
    ↓
handlers
```

Part 3 removes both outer semantic layers.

Target:

```text
protocol/source events
    ↓
persistent phalcom-modules workspace session
    ↓
one phalcom-semantic SemanticWorkspaceSession
    ↓
one Arc<phalcom-semantic::SemanticSnapshot>
    ↓
LSP RequestContext
    ↓
handlers
```

---

# 14. New persistent module workspace owner

Create:

```text
phalcom-modules/src/session.rs
```

with a persistent `WorkspaceModuleSession` (name may only change if the existing crate naming convention requires it; do not create two equivalent types).

Conceptual ownership:

```rust
pub struct WorkspaceModuleSession {
    universe: ProjectUniverse,
    provider: OverlaySourceProvider<FilesystemSourceProvider>,

    project_roots: BTreeMap<ProjectSourceIdentity, ResolvedProjectId>,

    modules_by_source: BTreeMap<SourceId, ModuleId>,
    sources_by_module: BTreeMap<ModuleId, WorkspaceSourceState>,

    standalone_projects: BTreeMap<SourceId, ResolvedProjectId>,

    linked: Option<Arc<LinkedProgram>>,
    generation: ModuleWorkspaceGeneration,
}
```

`WorkspaceModuleSession` is owned by compiler/module infrastructure, not by `phalcom-lsp`.

## 14.1 Reuse existing `phalcom-modules` foundations

Current repository already has:

- `ProjectUniverse`;
- `discover_owning_project`;
- `FilesystemSourceProvider`;
- `OverlaySourceProvider<P>`;
- `SourceOverlay`;
- `ParsedModuleUnit`;
- canonical `ModuleId`, `SourceId`, `SourceLocation`;
- linker/project graph products.

Part 3 composes these. Do not build another project resolver in `analysis_service.rs`.

---

# 15. Workspace source state

Use one canonical source record:

```rust
#[derive(Clone, Debug)]
pub struct WorkspaceSourceState {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub location: SourceLocation,
    pub revision: SourceRevision,
    pub text: Arc<str>,
    pub parsed: Arc<ParsedModuleUnit>,
    pub open_overlay: bool,
}
```

`SourceRevision` is source-lifecycle input identity. It is not the semantic DB revision and not the LSP document version.

Important invariants:

```text
same SourceId + ordinary text edit:
    stable project identity
    stable ModuleId
    new SourceRevision

rename/move across canonical module path:
    old ModuleId removed
    new ModuleId created/resolved

close editor:
    overlay removed
    canonical disk source becomes active

project.toml/root/dependency change:
    module workspace epoch may rebuild project resolution

ordinary body edit:
    does not rebuild ProjectUniverse
```

---

# 16. Source mutation API

Do not make `phalcom-modules` depend on `tower_lsp::Url`.

LSP converts protocol URIs to filesystem/source identities at the boundary.

A target API:

```rust
pub enum WorkspaceSourceMutation {
    SetOverlay {
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
    },
    RemoveOverlay {
        source: SourceId,
    },
    RefreshDisk {
        source: SourceLocation,
        revision: SourceRevision,
    },
    RemoveSource {
        source: SourceId,
    },
}

pub struct WorkspaceModuleUpdate {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub changed_modules: BTreeSet<ModuleId>,
    pub removed_modules: BTreeSet<ModuleId>,
    pub identity_changes: BTreeSet<ModuleId>,
}
```

Project-root/config changes use a separate explicit operation:

```rust
pub fn set_workspace_roots(
    &mut self,
    roots: &[PathBuf],
    dependency_provider: &dyn DependencyProvider,
) -> Result<WorkspaceModuleUpdate, ProjectError>;
```

Do not smuggle project-root changes through fake source mutations.

---

# 17. Stable standalone identity

A standalone file without `project.toml` currently requires synthetic project identity.

Part 3 requires:

```text
open standalone file A
    allocate synthetic project once

edit A 100 times
    same synthetic project identity
    same ModuleId when logical path unchanged

close/reopen A during same workspace epoch
    retain identity if source identity remains registered

delete A permanently
    release/remove its source mapping

new unrelated standalone file
    never accidentally inherit A's ModuleId
```

The current pattern of keeping standalone mappings in `StaticWorkspaceIdentity` moves into `WorkspaceModuleSession`.

Synthetic identity is module semantics and must not remain LSP semantic state.

---

# 18. Persistent project identity

For a project-backed file:

```text
SourceId
    ↓ discover owning project
ProjectSourceIdentity
    ↓ persistent ProjectUniverse
ResolvedProjectId
    ↓ canonical module path
ModuleId
```

`ProjectUniverse::load_root` already caches by project source identity. The module session keeps that universe alive.

Ordinary source edits must not repeatedly:

- rediscover/load the same manifest graph;
- allocate new `ResolvedProjectId`;
- reconstruct dependency aliases;
- recreate builtin import roots;
- recreate synthetic IDs.

Project configuration changes explicitly invalidate the affected project graph.

---

# 19. Overlay semantics

Open editor buffers have source precedence over disk without changing semantic identity.

Use the existing `OverlaySourceProvider`.

Lifecycle:

```text
didOpen:
    set overlay(module/source, editor text)

didChange:
    replace overlay text/revision

didClose:
    remove overlay
    refresh canonical disk text
    if disk == overlay semantic product:
        fingerprints may reuse products
    else:
        update affected module

watched-file change while open:
    update disk knowledge if desired
    do not override active overlay

watched-file change while closed:
    refresh disk source
```

An LSP open buffer must never be represented as a fake URI module distinct from its canonical project module.

---

# 20. Extend `SemanticWorkspaceSession`, do not replace it

Current `phalcom-semantic/src/session.rs` already owns:

```text
WorkspaceId
SemanticDb
TypeStore
base declarations
base hierarchy
base dispatch
base callable signatures
sources
source fingerprints
last snapshot
last-known-good snapshot
```

Part 3 adds/owns the persistent module session:

```rust
pub struct SemanticWorkspaceSession {
    workspace: WorkspaceId,
    modules: phalcom_modules::WorkspaceModuleSession,

    db: SemanticDb,
    store: TypeStore,
    // existing semantic state...
}
```

A high-level production update API should accept module/source lifecycle deltas and call the existing query/fingerprint machinery.

For example:

```rust
pub fn apply_workspace_changes(
    &mut self,
    changes: SemanticWorkspaceChanges,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> Result<SemanticWorkspacePublication, QueryOutcome<()>>;
```

The existing lower-level:

```rust
update(SemanticWorkspaceInput)
```

may remain as a compiler/test compatibility wrapper if useful, but **production LSP must not rebuild that input independently**.

---

# 21. Semantic workspace publication

Introduce one explicit publication result:

```rust
pub struct SemanticWorkspacePublication {
    pub snapshot: Arc<SemanticSnapshot>,
    pub invalidated: Arc<[QueryKey]>,
    pub recomputed: Arc<[QueryKey]>,
    pub stats: SemanticUpdateStats,
    pub effects: SemanticPublicationEffects,
}
```

Suggested effects:

```rust
#[derive(Clone, Debug, Default)]
pub struct SemanticPublicationEffects {
    pub diagnostics_changed: BTreeSet<ModuleId>,
    pub source_index_changed: BTreeSet<ModuleId>,
    pub formal_changed: BTreeSet<ModuleId>,
    pub advisory_changed: BTreeSet<ModuleId>,
    pub declaration_index_changed: bool,
    pub module_graph_changed: bool,
}
```

These effects are derived from compiler product fingerprints.

The LSP maps them to protocol refresh operations. The LSP does not decide whether semantic facts changed by comparing its own inferred structures.

---

# 22. One publication atom

A published snapshot must represent one coherent semantic revision:

```text
TypeStore
sources
module products
declaration surfaces
callable signatures
formal callable analyses
source semantic index
advisory products
diagnostics
presentation attachments
```

All must correspond to the same `SnapshotId`/semantic revision.

Forbidden:

```text
new advisory snapshot
+
old formal static_snapshot
```

or:

```text
new source index
+
old callable body facts
```

unless the compiler snapshot explicitly publishes a partial-state product whose dependencies prove the combination coherent.

The LSP must not assemble a “current” semantic world from independent timestamps.

---

# 23. Last-known-good semantics

Current `SemanticWorkspaceSession` already retains a last-known-good snapshot. Preserve that property.

Cancellation/budget/internal failure may prevent a candidate update from publishing.

Rules:

```text
candidate succeeds:
    atomically publish new snapshot

candidate cancelled/stale:
    discard candidate
    keep old published snapshot

candidate budget-exceeded:
    keep last-known-good publication
    expose analysis status separately

candidate has ordinary semantic diagnostics:
    publish it
    semantic errors are not infrastructure failure
```

Do not equate “program contains type errors” with “semantic snapshot is unusable”.

---

# 24. Open-document stale snapshot policy

A request must never map current document positions into semantic source ranges from a different source revision as though they were exact.

`RequestContext` target:

```rust
pub struct RequestContext {
    pub document: DocumentSnapshot,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub module: Option<phalcom_modules::ModuleId>,
    pub source_match: SourceMatch,
}

pub enum SourceMatch {
    Exact,
    Stale,
    Unmapped,
}
```

At handler entry:

1. pin open-document snapshot;
2. pin semantic snapshot exactly once;
3. resolve canonical module/source identity through snapshot/module source registry;
4. compare source revision/fingerprint;
5. never swap semantic snapshots mid-request.

## 24.1 Exact

All semantic position/target queries are allowed.

## 24.2 Stale

Allowed:

- lexical keyword hover;
- syntax recovery required to understand incomplete call/dot shape;
- protocol-level source text behavior;
- semantic answers that do not rely on stale source offsets and whose identity is independently stable.

Not allowed:

- using a stale source offset to select a formal expression;
- publishing old diagnostics against new text ranges;
- pretending old occurrence ranges belong to current text.

## 24.3 Unmapped

Use syntax-only behavior until the compiler session publishes canonical source identity.

---

# 25. Fast editor responsiveness without a second semantic engine

Part 3 must not trade correctness for a sluggish IDE.

The solution is staged compiler publication, not LSP-side reimplementation.

A source update can have two compiler-owned phases:

```text
Phase A — source/surface publication
    parse recovered AST
    update canonical module/source identity
    update lexical/source occurrence index
    update declaration shell/surface
    publish current source-oriented snapshot if coherent

Phase B — deep semantics
    formal body analysis
    advisory flow/interprocedural analysis
    diagnostics
    publish richer snapshot
```

The exact implementation may use one or two publication generations, but both are compiler snapshots and explicitly report completeness.

This supports immediate:

- token identity;
- local/source declarations;
- import/module structure;
- declaration completion;
- basic navigation;

while body inference catches up.

Do not recreate `shallow_receiver_classes`, an LSP scope graph, or LSP flow analysis as the “fast path”.

---

# 26. Production worker target

Retain the mature scheduling behavior in `phalcom-lsp/src/analysis_service.rs`:

- debounce;
- latest-wins coalescing;
- worker thread;
- cancellation epoch;
- workspace scan scheduling;
- open-buffer precedence;
- status/log notifications;
- source cache needed for protocol/source transport.

Replace semantic ownership inside the worker.

Target conceptual worker state:

```rust
struct AnalysisWorkerState {
    session: phalcom_semantic::SemanticWorkspaceSession,
    published: Arc<SemanticSnapshot>,

    // protocol/scheduler state only:
    source_epochs: BTreeMap<Url, u64>,
    open_documents: BTreeSet<Url>,
    config: WorkspaceAnalysisConfig,
}
```

There is no LSP `SemanticEngine`.

---

# 27. Delete `run_static_workspace_analysis(...)`

The current function is a transitional reconstruction seam.

After the new workspace lifecycle API exists:

```text
DELETE production run_static_workspace_analysis(...)
DELETE StaticWorkspacePublication
DELETE StaticWorkspaceIdentity
DELETE refresh_static_analysis(...) bridge
DELETE engine.set_static_analysis(...)
```

Tests that currently call `run_static_workspace_analysis` should be migrated to:

```rust
let mut session = SemanticWorkspaceSession::new();
session.apply_workspace_changes(...);
```

or a focused `WorkspaceModuleSession` harness.

Do not leave the old function under a “compatibility” name in production.

---

# 28. Remove nested `static_snapshot`

Current LSP `SemanticSnapshot` contains:

```rust
static_snapshot: Option<Arc<phalcom_semantic::SemanticSnapshot>>
```

Part 3 eliminates this entire composition.

The LSP reads the compiler snapshot directly.

Consequences:

- `formal_static_snapshot(...)` adapter disappears;
- `formal_static_module(...)` adapter disappears;
- `formal_binding_presentation_at(...)` scan disappears;
- `formal_expression_presentation_at(...)` scan disappears;
- `formal_callable_presentation(...)` name/selector bridge disappears;
- string-form formal type lookups disappear.

Compiler source-site/target indexes from Part 2 provide direct queries.

---

# 29. Backend target ownership

Current `Backend` owns:

- `WorkspaceIndex`;
- an LSP `SemanticDb`;
- `AnalysisService`;
- document/source caches;
- configuration.

After Part 3, semantic state is read through the compiler publication owned by the analysis service/session.

Target:

```rust
pub struct Backend {
    client: Client,
    documents: DocumentStore,

    analysis: AnalysisService,
    semantic: PublishedSemanticSnapshotHandle,

    workspace_roots: RwLock<Vec<Url>>,
    closed_sources: SourceCache,
    config: RwLock<ServerConfig>,

    // notification coalescing only:
    inlay_refresh: Arc<PublicationRefresh>,
    semantic_token_refresh: Arc<PublicationRefresh>,
}
```

`PublishedSemanticSnapshotHandle` may be an `RwLock<Arc<SemanticSnapshot>>` or another existing atomic publication abstraction. It has no semantic methods beyond reading the compiler snapshot.

No `WorkspaceIndex` semantic authority remains.

---

# 30. Diagnostics cutover

Current `combined_diagnostics_for` traverses:

```text
LSP advisory snapshot
 -> static_snapshot
 -> document map
 -> compiler diagnostics
```

Target:

```text
DocumentSnapshot
+
Arc<phalcom_semantic::SemanticSnapshot>
```

Algorithm:

1. render current syntax diagnostics from the current document parse;
2. if compiler source revision matches the document, append compiler semantic diagnostics for canonical `ModuleId`;
3. resolve secondary labels through compiler/module `SourceLocation`;
4. if compiler semantic snapshot is stale for the open buffer, suppress those stale semantic diagnostics until an exact publication arrives;
5. closed-file/workspace diagnostics use compiler source locations directly;
6. never produce semantic type diagnostics from advisory facts.

Semantic errors are published even when the snapshot is otherwise complete.

---

# 31. Hover cutover

`phalcom-lsp/src/hover.rs` currently mixes:

- lexical keyword scan;
- `WorkspaceIndex` declaration lookup;
- LSP advisory `InferredValue`;
- compiler `FormalPresentation`;
- Phaldoc source scan.

Target responsibilities:

Keep in LSP:

- keyword/token identification;
- Phaldoc rendering/harvesting if not moved into a source product;
- markdown composition.

Replace semantic lookup with:

```text
request.semantic.site_at(module, offset)
request.semantic.target_at(module, offset)
request.semantic.presentation_for_site(site)
request.semantic.declaration_source(target)
```

No callable-analysis scan and no string selector matching.

## 31.1 Hover examples

Ordinary established local:

```text
count: Int
```

Broader contract/current precision:

```text
cell: CellNum

Declared as `Number`.

Inferred from `CellNum.new()`.
```

Assumed annotation because value genuinely has no formal evidence:

```text
value: Payload

Declared type; no independent value evidence is available here.
```

Flow narrowing:

```text
user: Admin

Declared as `User`.

Narrowed in this branch by `user is Admin`.
```

Generic specialization:

```text
result: Result<User, NetworkError>

Specialized here from the call arguments.
```

Advisory-only:

```text
payload: Payload

Inferred from local flow.
```

No `≈`.

Unknown:

```text
value

Type unavailable: generic parameter `T` could not be inferred.
```

Native:

```text
System.print(_ value: Object) -> Unit

Writes `value` to standard output followed by a newline.

Return type from native signature.
```

---

# 32. Inlay-hint cutover

Current `phalcom-lsp/src/inlay_hints.rs` contains extensive separate formal/advisory traversal and explicit annotation scanning. Part 2 should already provide source sites and attachments; Part 3 makes those the only semantic input.

Target flow:

```text
visible source range
    ↓
compiler source semantic index
    ↓
declaration/binding/parameter/field/return sites
    ↓
formal/advisory presentation view
    ↓
LSP label
```

Visible labels:

```text
binding/field/parameter:
    : T

callable return:
     -> T
```

Tooltip examples:

```text
Inferred from `CellNum.new()`.
```

```text
Declared type.
```

```text
Inferred from call sites.
```

```text
Return type from native signature.
```

Tooltips are optional. Do not attach one merely to say “inferred by Phalcom”.

## 32.1 Hint suppression

Preserve and centralize these policies:

- explicit source type annotation suppresses duplicate type hint;
- explicit return annotation suppresses return hint;
- `Unknown` generally emits no type hint;
- `Dynamic` may be omitted if obvious from explicit annotation;
- heuristic advisory facts appear only under the existing `All`-style policy;
- obvious-literal suppression remains a presentation preference;
- invalid expressions with usable known type may still support a hint if that hint is not misleading.

The source index should expose “explicit annotation exists” so `inlay_hints.rs` does not recursively walk the AST solely to rediscover this fact.

---

# 33. Signature-help cutover

Current `signature_help.rs` correctly keeps syntax-only incomplete-call recovery, but it uses LSP `MemberSurface`, LSP advisory signature, compiler formal presentation, and visible `≈`.

Keep:

```rust
CallSite {
    name_range,
    receiver_range,
    selector candidate,
    active_parameter,
}
```

This is protocol syntax recovery.

Replace the semantic side with compiler canonical dispatch/callable presentation.

Target:

```text
syntax CallSite
    ↓
source-site/receiver compiler query
    ↓
CallableId
    ↓
CallableSemanticSignature
    ↓
formal/advisory parameter presentation
    ↓
plain signature string
```

Display:

```text
compute(_ value: String, with mapper: Function) -> Result<Data, Error>
```

No `≈`.

Do not render `Unknown` into every missing parameter slot when the better UX is to omit the type.

---

# 34. Completion cutover

Current `completion.rs` contains a large LSP-side “shallow” semantic reconstruction:

- receiver class inference;
- constructor detection;
- method-return scanning;
- field constructor tracking;
- argument constructor propagation;
- source `ModuleSurface` reconstruction;
- `WorkspaceIndex` fallback.

All of that semantic reconstruction must be removed after Part 2 compiler source/advisory products exist.

Keep only syntax recovery needed to find:

```text
receiver.
receiver.partial
incomplete call argument position
```

For an exact snapshot:

1. resolve source site/receiver through compiler source index;
2. use formal receiver if concrete;
3. otherwise use compiler advisory receiver shape if available;
4. query canonical compiler declaration/dispatch surfaces;
5. apply visibility;
6. render ordinary completion items.

For a stale snapshot, do not run a second semantic analyzer. Use source-oriented compiler products from a fast current publication when available; otherwise provide bounded lexical/global completions until the semantic publication catches up.

---

# 35. Completion evidence is not item decoration

If completion was enabled by advisory receiver evidence, do not add:

```text
≈
inferred
heuristic
advisory
```

to every member.

The completion list answers “what can I reasonably type here?” Its semantic provenance can be inspected via hover after insertion or through a focused detail field when genuinely useful.

This is a deliberate UX rule.

---

# 36. Definition cutover

Definition algorithm:

```text
site = snapshot.source_index.site_at(module, offset)
target = site.exact_target
    or, only if exact target absent, site.advisory_target

location = snapshot.declaration_location(target)
```

Exact compiler targets win.

Advisory target attachments from Part 2 may support useful navigation when formal dispatch is unavailable, but they must remain advisory data internally.

No `WorkspaceIndex::definition_info`.
No selector-string workspace scan.
No re-dispatch inside the LSP.

---

# 37. References cutover

Use Part 2's compiler reverse target index:

```text
SemanticTarget -> [SourceOccurrenceId]
```

Then project occurrences to `Location`.

For snapshot-local bindings, the target includes/guards the snapshot/source owner so an old binding ID cannot alias a new binding after edit.

No whole-workspace occurrence scan on each request.

---

# 38. Workspace symbols cutover

If `WorkspaceIndex` currently serves `workspace/symbol`, replace it with a compiler-owned declaration/source index.

The query product may contain a sorted search-friendly vector such as:

```rust
pub struct WorkspaceDeclarationEntry {
    pub target: SemanticTarget,
    pub name: Box<str>,
    pub kind: DeclarationKind,
    pub source: SemanticSourceSpan,
}
```

This is a derived snapshot product with a fingerprint.

It is not an LSP mutable semantic database.

---

# 39. Module/import completion cutover

Use `SemanticSnapshot::module_queries()` / canonical `phalcom-modules::ModuleQueryFacade` products.

Import completion must not scan workspace directories independently to create semantic import candidates when the canonical project/module universe is available.

Allowed LSP work:

- recover the partially typed import path;
- ask module query facade for roots/children/exports;
- convert to completion items.

Canonical module resolution remains in `phalcom-modules`.

---

# 40. Core/native source navigation

Part 3 preserves current virtual-source support but changes the semantic source of the target.

Definition of a native/core declaration uses:

```text
CallableId / DeclarationId
    ↓
canonical source provenance
    ↓
physical SourceLocation if available
or
phalcom:// virtual source location
```

The LSP content provider remains a protocol adapter.

Do not hardcode `phalcom://core` as the semantic identity of every native member.

---

# 41. Semantic token cutover

`semantic_tokens.rs` may keep lexer-driven classification for syntax that has no semantic identity.

For semantic role refinement:

```text
compiler occurrence index
    binding -> variable/parameter
    callable declaration/reference -> method
    field -> property
    class -> class
    operator/selector target -> appropriate custom token
```

When exact current semantic source products exist, do not reparse the AST solely to identify declaration/reference roles.

If the semantic snapshot is stale:

- lexer tokens remain safe;
- semantic role refinement may be omitted;
- do not use stale ranges.

This produces graceful degradation rather than a second analyzer.

---

# 42. Request-time purity

Every semantic handler must obey:

```text
request thread:
    read document snapshot
    read one semantic snapshot
    run pure indexed queries
    render protocol response
```

Forbidden on request path:

- filesystem reads for semantic resolution;
- project loading;
- module linking;
- mutable semantic DB queries that compute products;
- body type checking;
- advisory fixpoint solving;
- workspace scans;
- building class/module surfaces from AST;
- whole-workspace references scan.

Lazy compiler queries that are intentionally read-through and already concurrency-safe may be introduced later, but Part 3's cutover target is immutable published read queries.

---

# 43. Snapshot pinning

A handler must not do:

```rust
let a = semantic.snapshot();
// lookup receiver
let b = semantic.snapshot();
// lookup member
```

because another publication may occur between them.

It must do:

```rust
let request = RequestContext::new(
    document_snapshot,
    semantic_publisher.snapshot(),
    uri,
);
```

and use `request.semantic` for the entire operation.

Cross-file target/source lookup also uses that same snapshot.

---

# 44. Publication effects and client refresh

Map compiler publication effects to protocol notifications.

Examples:

```text
formal/advisory presentation changed:
    workspace/inlayHint/refresh

source semantic token role changed:
    workspace/semanticTokens/refresh

diagnostics changed:
    publish only affected module diagnostics

declaration/module surface changed:
    no global completion cache invalidation required if completion is snapshot-query based
```

Do not refresh inlays/tokens on every semantic generation if their product fingerprints are unchanged.

---

# 45. Professional diagnostics/hover interaction

Diagnostics and hover should complement one another.

Diagnostic:

```text
Type mismatch: expected `Int`, found `CellNum`.
```

Hover on binding:

```text
x: CellNum

Declared as `Int`.
```

The hover need not repeat the entire diagnostic prose unless the local conflict is otherwise unclear.

For a proven subtype contract:

```phalcom
let x: Number = CellNum.new()
```

hover:

```text
x: CellNum

Declared as `Number`.
```

No green checkmark and no “proof succeeded” boilerplate by default.

---

# 46. Formal/advisory disagreement presentation

The semantic model must retain disagreement because it is useful for debugging the analyzer, but ordinary UI should not alarm users unnecessarily.

Policy:

```text
formal Established(T)
advisory U, U != T
    primary = T
    normally hide U

formal Assumed(T)
advisory U, U != T
    primary = T
    hover MAY say:
        Declared as T.
        Local flow suggests U.
    never emit a compiler error from advisory disagreement

formal Unknown
advisory U
    primary UI may show U
    hover:
        Inferred from local flow/call sites.

formal Dynamic
advisory U
    primary formal semantics remain Dynamic where the UI asks for language type;
    completion may still use U as a practical candidate source if policy allows
```

Do not silently turn advisory disagreement into proof evidence.

---

# 47. Do not expose internal confidence ranking as a score

Current advisory `Confidence::{Exact, Flow, Interprocedural, Heuristic}` remains useful internally.

Do not render:

```text
Confidence: flow
Confidence: 0.8
Confidence: exact
```

as ordinary hover prose.

Translate to cause:

```text
Inferred from local flow.
Inferred from call sites.
Best-effort inference from usage.
```

Developers care about why, not an analyzer taxonomy.

---

# 48. Explanation/provenance integration

Part 1/2 explanation data should support richer hover without making hover dependent on raw explanation graphs.

Add a bounded presentation projection from explanation/provenance:

```rust
pub struct EvidenceSummary {
    pub kind: EvidenceSummaryKind,
    pub source: Option<SemanticSourceSpan>,
    pub related_target: Option<SemanticTarget>,
    pub description: Option<Box<str>>,
}
```

This is a read-only summary generated by `phalcom-semantic`, not LSP graph traversal.

Examples:

```text
Constructor result
Native signature
Declared annotation
Flow narrowing
Generic specialization
Call-site evidence
Pattern decomposition
```

The LSP maps these to prose.

Do not ship an unbounded proof tree in every hover response.

---

# 49. Hover evidence source links

Where the LSP client supports markdown command/source links safely, evidence may expose a related source target:

```text
Inferred from `CellNum.new()`.
```

The referenced `CellNum.new()` can be navigable through canonical target location.

This is optional UI enrichment. The semantic target must come from compiler provenance, not string parsing of the evidence sentence.

---

# 50. Performance architecture after cutover

The important performance property is not merely “fast enough on one laptop.”

The architecture must prove bounded recomputation.

Ordinary body edit:

```text
parse changed module
update source-index products for changed module
invalidate changed callable/body query
propagate through explicit semantic dependency closure only
recompute affected advisory contribution sources only
reuse unchanged project/link/declaration products
```

Must not:

```text
reload project.toml
reallocate project IDs
relink every module
reanalyze every callable
rebuild workspace symbol index from raw files
re-run LSP advisory solver
```

---

# 51. Structural performance counters

Extend `SemanticUpdateStats` as necessary:

```rust
pub struct SemanticUpdateStats {
    pub modules_recomputed: usize,
    pub callables_recomputed: usize,
    pub callables_reused: usize,

    pub project_graph_rebuilt: bool,
    pub modules_relinked: usize,
    pub source_indexes_recomputed: usize,
    pub advisory_sources_recomputed: usize,
    pub advisory_callables_recomputed: usize,
}
```

Exact field names may be adjusted to current metrics conventions, but tests need equivalent observability.

CI gates assertions such as:

```text
body-only edit:
    project_graph_rebuilt == false
    modules_relinked == 0 or bounded to semantically affected module products
    unrelated callable recomputation == 0

comment-only edit:
    formal callable product reuse where semantic fingerprint unchanged
    source range/index product updates only where required

project.toml dependency edit:
    project graph rebuild allowed
```

---

# 52. Cold-vs-incremental parity

Given the same final source universe:

```text
cold session:
    load final sources once

incremental session:
    load initial sources
    apply edit sequence
    reach same final text
```

They must agree on semantically observable products:

- canonical module identities where lifecycle rules imply same project/source identity;
- declaration/callable/field targets;
- callable signatures;
- formal type knowledge;
- diagnostics;
- source target identity;
- advisory shape products;
- references;
- completion surfaces;
- presentation primary types.

Revision numbers and transient recomputation counters may differ.

This is a final single-world proof.

---

# 53. Open/change/close lifecycle tests

Required integration sequence:

```text
open project file with disk version A
    compiler sees A overlay

edit to B
    compiler sees B
    stable ModuleId

request hover/completion
    snapshot B only

close
    overlay removed
    disk A becomes active

compiler updates back to A
    same canonical ModuleId

reopen B
    overlay B active
```

Assertions:

- no synthetic URI module allocation for project file;
- no duplicate source entries;
- stale B snapshot is not used for A ranges after close;
- same project identity survives.

---

# 54. Delete/rename lifecycle tests

Delete:

```text
module A exists
references target A
delete source
    remove source/module product
    reverse target entries removed
    importers invalidated appropriately
```

Rename/move:

```text
old/path.ph -> new/path.ph
```

If the logical module path changes, treat as remove+add identity transition. Do not preserve `ModuleId` merely because an editor supplied a rename event.

If physical spelling changes without changing canonical logical identity under the module naming rules, follow `phalcom-modules` canonical source identity rules.

---

# 55. Project configuration lifecycle tests

Change:

```text
project.toml dependency alias/source root/entry
```

Allowed:

- rebuild affected project/module universe;
- replace linked products;
- invalidate import-dependent semantic products.

Required:

- no stale `ResolvedProjectId` mapping reused incorrectly;
- old snapshot remains immutable for in-flight requests;
- new snapshot publishes atomically;
- open overlays remap to the new canonical module identity if project semantics changed.

---

# 56. Cancellation/latest-wins tests

Retain the mature LSP worker behavior but verify it against compiler publications.

Sequence:

```text
edit A -> starts candidate 1
edit B -> increments worker epoch
candidate 1 observes cancellation
candidate 1 discarded
candidate 2 completes
only candidate 2 publishes
```

Assert:

- no partial candidate reaches request readers;
- last-known-good snapshot remains available until candidate 2;
- TypeStore/session identity remains valid;
- source/advisory/formal products all come from the same published candidate.

---

# 57. Semantic errors still publish

Given:

```phalcom
let x: Int = CellNum.new()
```

the semantic session publishes:

```text
diagnostic: mismatch
binding current: CellNum
binding contract: Int
source targets
completion/navigation products
```

The LSP must not revert to the previous good source merely because the current program is semantically invalid.

“Last known good” is for infrastructure/cancellation failure, not ordinary type errors.

---

# 58. Practical IDE golden matrix

Add/extend `examples/ide-golden` and Rust integration tests with exact user-visible assertions.

Mandatory cases:

1. inferred established binding shows `: CellNum`, no `≈`;
2. advisory-only binding shows `: Payload`, no `≈`;
3. explicit annotation suppresses inlay;
4. flow-narrowed hover shows narrowed type + concise origin;
5. broad declaration/current precision hover shows both types;
6. refuted declaration hover keeps actual type;
7. assumed annotation hover explains assumption only when useful;
8. generic specialization hover shows specialized type;
9. native signature hover says return from native signature;
10. `System.print` tail method shows ` -> Unit`;
11. signature help has no `≈`;
12. completion list has no advisory decoration;
13. unknown generic does not fabricate a type hint;
14. formal/advisory disagreement prefers formal;
15. heuristic advisory is hidden under Stable policy;
16. Phaldoc remains prominent and is not buried below analyzer boilerplate.

---

# 59. `System.print` end-to-end golden

Fixture:

```phalcom
class Greeter {
  greet(_ name: String) {
    System.print(name)
  }
}

let greeter = Greeter.new()
greeter.greet("Ada")
```

Expected semantic facts:

```text
System.print(_) -> Established Unit
Greeter.greet(_) -> Established Unit
greet call -> Established Unit
```

Expected IDE:

```text
Greeter.greet(_ name: String) -> Unit
```

return inlay:

```text
 -> Unit
```

No occurrence of `Option` in inferred return presentation for `greet`.

No occurrence of `≈`.

This test must fail if advisory `Option` leaks into formal return inference or presentation.

---

# 60. Similar fixed-return regressions

Add a table-driven semantic integration suite covering at least these categories:

```text
native fixed Unit return
native fixed ordinary nominal return
native Never
native receiver return
native argument(i) return
source declared fixed return
constructor Self return
generic specialized return
```

For each:

```text
formal call result has correct TypeKnowledge
caller tail return uses formal result
advisory summary cannot weaken/replace it
hover/inlay use plain type label
```

This is the general protection requested by the `System.print` bug.

---

# 61. Delete LSP semantic authority

After Part 2 migration parity and Part 3 consumer cutover, remove the duplicate semantic implementation.

Expected deletion set includes the old semantic implementation files under:

```text
phalcom-lsp/src/semantic/
```

such as:

```text
analyzer.rs
callable.rs
dispatch.rs
engine.rs
facts.rs          // only after compiler-owned equivalent is in Part 2
flow.rs
ids.rs
infer.rs
invalidation.rs
module_graph.rs
occurrence.rs
query.rs
scope.rs
snapshot.rs
surface.rs
```

If Part 2 already deleted/moved a subset, do not recreate them.

The final `phalcom-lsp` may have a small module named `semantic` only if it is a pure re-export/protocol adapter with no semantic state or algorithms. Prefer direct `phalcom_semantic` imports where readable.

---

# 62. Delete `WorkspaceIndex` semantic authority

Audit `phalcom-lsp/src/index.rs`.

After compiler source/declaration/reverse-target indexes serve:

- definition;
- references;
- workspace symbols;
- class/member lookup;
- completion source surfaces;

remove `WorkspaceIndex`.

If a residual text-search index is retained for a non-semantic feature, rename it to reflect that narrow role and ensure it stores no semantic identity/dispatch truth.

Do not retain the old class/member selector index “just in case”.

---

# 63. Keep legitimate LSP-owned concerns

Single-world does not mean “move everything into the compiler”.

These remain appropriate in LSP:

```text
DocumentStore
LineIndex
LSP Position/Range conversion
keyword hover prose
Phaldoc markdown rendering
incomplete-call lexical recovery
dangling-dot lexical recovery
client capability negotiation
workspace folder protocol handling
debounce/scheduling
analysis status/log transport
configuration parsing
virtual document content provider
semantic-token wire encoding
completion item/snippet formatting
```

The test is: does this code decide what the program means? If yes, it belongs in compiler/module semantics.

---

# 64. File-level implementation map

## 64.1 Create

```text
phalcom-modules/src/session.rs
phalcom-modules/tests/workspace_session.rs

phalcom-lsp/src/presentation.rs

phalcom-core/tests/native_contracts.rs
phalcom-semantic/tests/trusted_return_contracts.rs
phalcom-lsp/tests/professional_semantic_presentation.rs
phalcom-lsp/tests/single_world_cutover.rs
```

Register integration tests explicitly where the crate uses `autotests = false`.

## 64.2 Modify — modules/compiler

```text
phalcom-modules/src/lib.rs
phalcom-modules/src/project.rs          // only helpers required by persistent session
phalcom-modules/src/source.rs           // overlay/session helpers, no duplicate provider

phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/db/key.rs          // only if lifecycle/source products need query keys
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/presentation.rs
phalcom-semantic/src/types/native.rs    // tests/contract preservation if needed
phalcom-semantic/src/checker/call.rs    // formal fixed-return precedence if Part 1 did not already finish it
phalcom-semantic/src/checker/body.rs     // tail return propagation if needed

phalcom-core/src/primitive/system.rs
docs/spec/current/system.md
```

## 64.3 Modify — LSP

```text
phalcom-lsp/src/analysis_service.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/request_context.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/inlay_hints.rs
phalcom-lsp/src/signature_help.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/diagnostics.rs
phalcom-lsp/src/import_completion.rs
phalcom-lsp/src/lib.rs / main module registration as required
phalcom-lsp/Cargo.toml
```

## 64.4 Delete after parity

```text
phalcom-lsp/src/semantic/* duplicate implementation
phalcom-lsp/src/index.rs                 // if no legitimate residual consumer
```

Deletion occurs only after compiler-owned replacement tests are green.

---

# 65. Implementation Task 1 — Lock professional presentation policy in tests

**Files:**

```text
phalcom-lsp/tests/professional_semantic_presentation.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/inlay_hints.rs
phalcom-lsp/src/signature_help.rs
```

Before broad cutover, add failing tests expressing the target UI.

Assertions:

```text
no visible `≈`
no `Confidence:`
no `Observed type:`
no `Observed return:`
plain canonical type labels
evidence wording only in tooltip/hover
```

Include formal and advisory sources so visual equality is intentional.

Do not “fix” tests by dropping advisory hints entirely.

---

# 66. Implementation Task 2 — Repair `System.print` canonical contract

**Files:**

```text
phalcom-core/src/primitive/system.rs
docs/spec/current/system.md
phalcom-core/tests/native_contracts.rs
```

Change `System.print`:

```text
returns Object -> Unit
types (Object)->Object -> (Object)->Unit
runtime none_value -> unit_value
stale docs pass-through result -> Unit
```

Also correct the confirmed `System.gc` metadata mismatch:

```text
returns Object -> None
types ()->Object -> ()->None
runtime remains none_value
```

Test runtime and native metadata for both.

Then run the native catalog coherence audit and address additional concrete mismatches discovered under §11.

---

# 67. Implementation Task 3 — Prove trusted fixed returns end-to-end

**Files:**

```text
phalcom-semantic/src/types/native.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/tests/trusted_return_contracts.rs
```

Verify/implement the precedence in §10.

Required red/green regression:

```phalcom
class Demo {
  run() {
    System.print("hello")
  }
}
```

Expected:

```text
call expression = Established(Unit)
caller return = Established(Unit)
```

Inspect evidence/provenance so hover can identify native signature support.

Do not special-case the selector string `"print(_)"` inside the checker. The only `System.print` special case is correcting its metadata/runtime declaration; semantic propagation must be generic.

---

# 68. Implementation Task 4 — Add persistent `WorkspaceModuleSession`

**Files:**

```text
phalcom-modules/src/session.rs
phalcom-modules/src/lib.rs
phalcom-modules/tests/workspace_session.rs
```

Compose existing `ProjectUniverse`, providers, source identities and linker.

Tests:

```text
stable project ID across edit
stable module ID across edit
stable standalone synthetic identity
overlay precedence
close overlay restores disk
remove source deletes module mapping
project config change rebuilds project graph
ordinary body edit does not
```

No LSP types in this crate.

---

# 69. Implementation Task 5 — Integrate module session into semantic session

**Files:**

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/db/fingerprint.rs
```

Add high-level incremental lifecycle API.

Keep existing stable `TypeStoreId` semantics.

Publish `SemanticWorkspacePublication` with effects.

Tests:

```text
source mutation -> bounded semantic invalidation
removed module -> reverse closure invalidated
cancelled update -> last publication retained
semantic errors -> current invalid program still publishes
```

---

# 70. Implementation Task 6 — Replace LSP static workspace reconstruction

**Files:**

```text
phalcom-lsp/src/analysis_service.rs
```

Route worker events directly into compiler session lifecycle.

Delete production:

```text
run_static_workspace_analysis
StaticWorkspaceIdentity
StaticWorkspacePublication
refresh_static_analysis bridge
```

Retain worker debounce/cancellation/status/logging.

Tests must demonstrate the worker reuses the same compiler session across at least two edits and does not allocate a new project universe/type store on each edit.

---

# 71. Implementation Task 7 — Publish compiler snapshot directly

**Files:**

```text
phalcom-lsp/src/analysis_service.rs
phalcom-lsp/src/backend.rs
phalcom-lsp/src/request_context.rs
```

Replace LSP semantic DB publication with one `Arc<phalcom_semantic::SemanticSnapshot>` handle.

Remove nested `static_snapshot`.

`RequestContext` pins one compiler snapshot.

Test concurrent publication while a request holds the older `Arc`: the request must finish coherently against the old snapshot.

---

# 72. Implementation Task 8 — Cut diagnostics to compiler snapshot

**Files:**

```text
phalcom-lsp/src/backend.rs
phalcom-lsp/src/diagnostics.rs
phalcom-lsp/tests/single_world_cutover.rs
```

Remove LSP semantic diagnostic composition.

Keep current syntax errors.

Use canonical module source mapping.

Test stale open-buffer suppression and exact-version publication.

---

# 73. Implementation Task 9 — Add central presentation renderer

**Files:**

```text
phalcom-lsp/src/presentation.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/inlay_hints.rs
phalcom-lsp/src/signature_help.rs
```

Implement §§3–7.

Tests should operate on compiler presentation/site views directly where possible so wording rules do not require a live worker.

No semantic inference in this module.

---

# 74. Implementation Task 10 — Cut hover to source-site views

**Files:**

```text
phalcom-lsp/src/hover.rs
phalcom-lsp/src/backend.rs
```

Remove:

- `WorkspaceIndex` semantic lookup;
- LSP `InferredValue` lookup;
- formal callable-analysis scans;
- selector/name bridge.

Retain keyword/Phaldoc behavior.

Add exact hover goldens from §31.

---

# 75. Implementation Task 11 — Cut inlay hints and signature help

**Files:**

```text
phalcom-lsp/src/inlay_hints.rs
phalcom-lsp/src/signature_help.rs
```

Inlays enumerate compiler source sites, not AST-semantic reconstruction.

Signature help retains syntax `CallSite` recovery only.

Remove all visible advisory markers.

System.print regression must pass here.

---

# 76. Implementation Task 12 — Cut completion to canonical compiler surfaces

**Files:**

```text
phalcom-lsp/src/completion.rs
phalcom-lsp/src/backend.rs
```

Delete semantic “shallow” receiver inference once compiler source/advisory products replace it.

Keep only range/partial-token recovery.

Test:

- formal receiver;
- advisory-only receiver;
- union receiver;
- `self`;
- `super`;
- class object;
- native/core receiver;
- incomplete dangling dot;
- current-buffer recovery while deep analysis is pending.

No `≈` or per-item “inferred” decoration.

---

# 77. Implementation Task 13 — Cut navigation/references/workspace symbols

**Files:**

```text
phalcom-lsp/src/backend.rs
phalcom-lsp/src/index.rs
```

Switch definition/references/workspace symbols to compiler source/reverse target/declaration indexes.

Once all consumers are gone, delete `WorkspaceIndex`.

Test cross-module target identity and snapshot-local binding references.

---

# 78. Implementation Task 14 — Cut module completion and source navigation

**Files:**

```text
phalcom-lsp/src/import_completion.rs
phalcom-lsp/src/backend.rs
```

Use compiler snapshot `ModuleQueryProducts`/`ModuleQueryFacade`.

Use canonical source provenance for core/native virtual source.

No filesystem semantic discovery on request path.

---

# 79. Implementation Task 15 — Cut semantic-token role refinement

**Files:**

```text
phalcom-lsp/src/semantic_tokens.rs
```

Keep lexer tokenization.

Replace semantic AST refinement with compiler occurrences when exact snapshot exists.

Stale fallback = lexical classification only.

Test declaration/reference roles across edit publication.

---

# 80. Implementation Task 16 — Delete duplicate LSP semantic system

Delete remaining `phalcom-lsp/src/semantic` implementation and residual old types.

Fix imports directly to compiler/module types.

Run forbidden-symbol audit in §86.

Do not keep a shadow compatibility layer that still owns mutable semantic state.

---

# 81. Implementation Task 17 — Add lifecycle/parity/performance acceptance suites

**Files:**

```text
phalcom-modules/tests/workspace_session.rs
phalcom-semantic/tests/type_store_revisions.rs
phalcom-semantic/tests/semantic_single_world.rs
phalcom-lsp/tests/single_world_cutover.rs
phalcom-lsp/tests/performance.rs
examples/ide-golden/...
```

Cover §§50–60.

Structural assertions are required in CI; wall-clock measurements are supplementary.

---

# 82. Implementation Task 18 — Remove obsolete docs/tests and record architecture

Update architecture documentation to show one semantic world.

Remove tests whose only purpose was validating the dual-world bridge.

Do not delete valuable behavioral test cases—port them to the canonical API.

Update comments in `backend.rs`, `analysis_service.rs`, hover/completion modules so they no longer describe `WorkspaceIndex` or LSP semantic engine as semantic authority.

---

# 83. Test taxonomy

Use four layers.

## 83.1 Module lifecycle unit/integration tests

Prove identity, overlay and project session behavior without semantic checker complexity.

## 83.2 Semantic integration tests

Prove formal/advisory/source products and incrementality with direct `SemanticWorkspaceSession`.

## 83.3 LSP renderer tests

Pure input -> hover/inlay/signature/completion output.

These should be fast and precise.

## 83.4 End-to-end LSP tests

Open/change/request/publish behavior through the server.

Do not rely only on end-to-end tests to diagnose semantic defects.

---

# 84. Negative semantic tests

The final suite must explicitly prove what no longer happens.

Examples:

```text
advisory String does not refute formal Int
advisory Option does not replace native Unit
Assumed Int is not displayed as a special pseudo-type
semantic error does not erase established current type
stale snapshot does not publish semantic diagnostic ranges
old BindingId cannot resolve in new snapshot
LSP cannot create ModuleId from raw URI as semantic identity
completion cannot invoke LSP-side flow analyzer
```

---

# 85. UX forbidden-pattern audit

After migration:

```bash
rg -n '≈|Observed type:|Observed return:|Confidence:|Inferred runtime value:' phalcom-lsp/src phalcom-lsp/tests
```

Expected production result:

```text
no matches under phalcom-lsp/src
```

Tests may contain forbidden strings only in explicit negative assertions such as:

```rust
assert!(!hover.contains("≈"));
```

Also search:

```bash
rg -n 'These .* inferred by Phalcom|inferred by Phalcom' phalcom-lsp/src
```

Expected: no boilerplate.

---

# 86. Single-world forbidden-symbol audit

After Part 3:

```bash
rg -n \
'run_static_workspace_analysis|StaticWorkspaceIdentity|StaticWorkspacePublication|static_snapshot|formal_static_snapshot|formal_binding_presentation_at|formal_expression_presentation_at' \
phalcom-lsp/src
```

Expected: zero.

Search:

```bash
rg -n \
'struct SemanticEngine|struct SemanticDb|struct ValueShape|struct ScopeGraph|enum SemanticTarget|struct CallableId|struct ClassId|struct FieldId' \
phalcom-lsp/src/semantic phalcom-lsp/src
```

Any match requires manual review. Protocol DTOs with unrelated names do not count; duplicate semantic definitions do.

Search:

```bash
rg -n 'ModuleId::new\(.*uri.*to_string|build_module_surface|WorkspaceIndex' phalcom-lsp/src
```

Expected semantic-authority uses: zero.

---

# 87. Native-contract forbidden-pattern audit

For `System.print`:

```bash
rg -n 'System.*print|print\(_\)' phalcom-core/src/primitive/system.rs docs/spec/current/system.md
```

Verify all canonical return declarations say `Unit`.

Add a test that fails if the generated native surface for `System.print(_)` is anything other than canonical Unit.

Search catalog-wide for suspicious broad returns where runtime body returns a known absence/unit sentinel. Review rather than mass-rewrite.

---

# 88. Part 3 completion gate

Every item below is mandatory.

1. Part 1 release gate passes.
2. Part 1 corrections/amendments are implemented.
3. Part 2 release gate passes.
4. `WorkspaceModuleSession` owns persistent project/source/module lifecycle.
5. Ordinary source edits retain project identity.
6. Ordinary source edits retain canonical ModuleId when logical identity is unchanged.
7. Standalone synthetic identity is stable across edits.
8. Open source overlays have precedence over disk.
9. Closing an overlay restores disk source without inventing a new module identity.
10. Project/root/dependency changes use an explicit project lifecycle invalidation path.
11. `SemanticWorkspaceSession` is the sole semantic session owner.
12. One TypeStore/TypeStoreId is retained across ordinary workspace revisions.
13. Production `run_static_workspace_analysis` is deleted.
14. `StaticWorkspaceIdentity` is deleted.
15. Nested LSP `static_snapshot` publication is deleted.
16. The LSP worker publishes one compiler `Arc<SemanticSnapshot>`.
17. A request pins exactly one semantic snapshot.
18. Source-position semantic queries require source-revision coherence.
19. Stale semantic diagnostics are not rendered against current open-buffer ranges.
20. Semantic errors still publish current semantic products.
21. Cancelled/stale candidate updates never publish.
22. Last-known-good publication survives infrastructure failure.
23. Formal and advisory facts remain distinct internally.
24. Advisory facts cannot emit hard type diagnostics.
25. Advisory facts cannot replace Established formal knowledge.
26. Advisory facts cannot upgrade formal assumptions/proofs.
27. Ordinary advisory type labels contain no `≈`.
28. Inlay hints contain no `≈`.
29. Signature help contains no `≈`.
30. Completion items contain no advisory glyph/status decoration.
31. Production hover contains no `Confidence:` taxonomy.
32. Production hover contains no `Observed type:`/`Observed return:` boilerplate.
33. Hover primary line uses canonical ordinary Phalcom spelling.
34. Hover shows declared/current distinction when materially useful.
35. Hover explains Assumed evidence when materially useful.
36. Hover explains flow narrowing when materially useful.
37. Hover explains generic specialization when materially useful.
38. Hover can explain advisory-only inference without mathematical notation.
39. Phaldoc remains prominent.
40. `System.print(_)` native metadata returns Unit.
41. `System.print(_)` Rust implementation returns `vm.unit_value()`.
42. `docs/spec/current/system.md` documents Unit return for print.
43. Compiler native import establishes Unit for `System.print`.
44. A `System.print` call expression is Established Unit.
45. A method with `System.print` as its normal tail is Established Unit.
46. Such a method's inlay/hover never reports Option.
47. Trusted fixed-return formal contracts take precedence over advisory summaries generally.
48. Native metadata self-consistency audit passes.
49. `System.gc` native metadata is reconciled to the canonical documented `None` return without changing it to `Unit`.
50. Diagnostics consume the compiler snapshot directly.
51. Hover consumes compiler source-site/presentation views directly.
52. Inlay hints consume compiler source sites directly.
53. Signature help resolves canonical compiler callable signatures.
54. Completion consumes compiler receiver/surface/advisory products.
55. Definition consumes compiler target/location indexes.
56. References consume compiler reverse target index.
57. Workspace symbols consume compiler declaration index.
58. Import/module completion consumes `ModuleQueryFacade`.
59. Core/native navigation consumes canonical source provenance.
60. Semantic token semantic refinement consumes compiler occurrences.
61. No semantic handler performs filesystem resolution on request path.
62. No semantic handler runs formal analysis on request path.
63. No semantic handler runs advisory solving on request path.
64. No semantic handler rebuilds declaration/module surfaces from AST.
65. Duplicate LSP semantic engine/database is deleted.
66. Duplicate LSP semantic IDs are deleted.
67. Duplicate LSP scope/occurrence/dispatch/module graph/advisory solver are deleted.
68. `WorkspaceIndex` semantic authority is deleted.
69. Cold and incremental final semantic products pass parity tests.
70. Open/change/close lifecycle tests pass.
71. Delete/rename lifecycle tests pass.
72. Project configuration lifecycle tests pass.
73. Cancellation/latest-wins tests pass.
74. Concurrent old-snapshot request immutability test passes.
75. Body-only edit structural counters show no project-universe rebuild.
76. Unrelated callables remain reused after isolated body edit.
77. Presentation-only/semantic-token refreshes are fingerprint-driven.
78. `cargo check --workspace` passes.
79. `cargo test -p phalcom-modules` passes.
80. `cargo test -p phalcom-semantic` passes.
81. `cargo test -p phalcom-core` passes.
82. `cargo test -p phalcom-lsp` passes.
83. IDE golden acceptance tests pass.
84. UX forbidden-pattern audit is manually reviewed.
85. Single-world forbidden-symbol audit is manually reviewed.
86. Native-contract audit is manually reviewed.
87. A reviewer can point to exactly one owner for project/module identity.
88. A reviewer can point to exactly one owner for formal semantics.
89. A reviewer can point to exactly one owner for advisory semantics.
90. A reviewer can point to exactly one immutable semantic snapshot consumed by all semantic LSP requests.

Part 3 is complete only when all 90 gates are true.

---

# 89. Recommended commit sequence

Keep the implementation reviewable:

```text
1. test(lsp): lock practical semantic presentation contract
2. fix(core): make System.print canonical Unit
3. test(semantic): prove trusted fixed returns before advisory fallback
4. modules: add persistent workspace module session
5. semantic: consume persistent module session
6. semantic: publish lifecycle effects with one snapshot
7. lsp: replace static workspace reconstruction with compiler session
8. lsp: publish compiler snapshot directly
9. lsp: cut diagnostics to compiler snapshot
10. lsp: centralize practical semantic presentation
11. lsp: cut hover to canonical source-site views
12. lsp: cut inlay and signature help
13. lsp: cut completion to compiler surfaces
14. lsp: cut definition/references/workspace symbols
15. lsp: cut module/source navigation and semantic token refinement
16. lsp: delete duplicate semantic engine and WorkspaceIndex authority
17. test: add lifecycle/cold-incremental/performance acceptance gates
18. docs: record final single-world architecture
```

The deletion commit should be mostly deletion/import rewiring because replacement consumers are already green.

---

# 90. Final architecture after Part 3

```text
                        editor / LSP client
                               │
                 protocol events / requests
                               │
                               ▼
                    phalcom-lsp adapters
       documents · ranges · debounce · rendering · notifications
                               │
                         source mutation
                               │
                               ▼
                    phalcom-modules
                WorkspaceModuleSession
      ProjectUniverse · ModuleId · SourceId · overlays · linking
                               │
                     canonical module update
                               │
                               ▼
                    phalcom-semantic
                SemanticWorkspaceSession
       SemanticDb · TypeStore · formal checker · advisory analysis
          source index · targets · diagnostics · presentation views
                               │
                               ▼
                 Arc<SemanticSnapshot>
                one coherent semantic world
                               │
              ┌────────────────┼─────────────────┐
              ▼                ▼                 ▼
         diagnostics      hover/inlays      completion/nav/
                                             tokens/signature
              │                │                 │
              └────────────────┴─────────────────┘
                               │
                          pure rendering
                               │
                               ▼
                            client
```

There is no semantic round-trip back through an LSP-owned analyzer.

---

# 91. What the IDE should feel like after all three parts

The internal compiler can answer questions like:

```text
Is this fact Established or Assumed?
What evidence established it?
Was it produced by native signature, constructor semantics, generic inference or flow?
Is there causal invalidity?
Is a second advisory shape available?
```

The developer normally sees:

```text
x: CellNum

Declared as `Number`.

Inferred from `CellNum.new()`.
```

not:

```text
x
Formal type: CellNum
Authority: Proven
Evidence status: Established
Observed type: ≈ CellNum
Confidence: Exact
```

The additional machinery exists to make the answer trustworthy and explainable, not to make ordinary programming feel ceremonial.

For common code, the ideal hover can be only:

```text
count: Int
```

For a useful surprise:

```text
user: Admin

Declared as `User`.

Narrowed in this branch by `user is Admin`.
```

For a native effect:

```text
System.print(_ value: Object) -> Unit

Writes `value` to standard output followed by a newline.

Return type from native signature.
```

For uncertain but useful editor intelligence:

```text
payload: Payload

Inferred from call sites.
```

That is the intended Phalcom IDE contract: concise first, explainable on demand, and semantically honest underneath.

---

# 92. Repository grounding record

This specification was grounded against `aureat/phalcom-lang` `main` commit:

```text
c36586619b1bf8f93429377b31425888b77f7df1
feat(semantic): establish epistemic correctness foundations
```

The following inspected implementation facts materially shaped the plan.

## 92.1 Part 1 model has begun landing

`phalcom-semantic/src/types/evidence.rs` now contains:

```text
EvidenceStatus::Established
EvidenceStatus::Assumed

EvidenceOrigin::Syntax
EvidenceOrigin::DeclarationSemantics
EvidenceOrigin::ConstructorSemantics
EvidenceOrigin::CallableSignature
EvidenceOrigin::NativeSignature
EvidenceOrigin::DeveloperAnnotation
EvidenceOrigin::GenericInference
EvidenceOrigin::Flow
EvidenceOrigin::ContextualDerivation
EvidenceOrigin::PatternDecomposition
```

This makes a rich-but-practical hover projection possible without exposing raw enums.

## 92.2 Native import already establishes native signatures

`phalcom-semantic/src/types/native.rs` lowers native metadata to `TypeKnowledge` with established `NativeSignature` origin and registers canonical callable semantic signatures.

Therefore `System.print` does not need an LSP exception. It needs correct native metadata and generic formal call-result propagation.

## 92.3 `System.print` currently has inconsistent contracts

`phalcom-core/src/primitive/system.rs`:

```text
metadata return: Object
runtime normal result: vm.none_value()
```

`docs/spec/current/system.md`:

```text
documented return: argument/pass-through
```

Part 3's target `Unit` is a deliberate semantic correction and must update runtime + metadata + docs together.

## 92.4 LSP presentation currently exposes analyzer taxonomy

`phalcom-lsp/src/hover.rs` currently emits:

```text
Formal type:
Observed type: ≈ ...
Confidence: ...
```

and equivalent return wording.

`phalcom-lsp/src/inlay_hints.rs` currently emits `≈ T` in advisory labels and tests.

`phalcom-lsp/src/signature_help.rs` currently inserts `≈ T` for advisory parameter/return presentation.

These are explicit Part 3 migration targets.

## 92.5 LSP still owns a dual publication path

`phalcom-lsp/src/analysis_service.rs` still contains:

```text
StaticWorkspaceIdentity
run_static_workspace_analysis
SemanticWorkspaceSession nested inside the LSP identity wrapper
```

and publishes compiler static analysis into the old LSP semantic snapshot.

This is the central SC-15 deletion target.

## 92.6 Persistent module primitives already exist

`phalcom-modules` already provides:

```text
ProjectUniverse
discover_owning_project
FilesystemSourceProvider
OverlaySourceProvider
SourceOverlay
ParsedModuleUnit
ModuleId
SourceId
SourceLocation
```

Part 3 composes these into a persistent session rather than replacing them.

## 92.7 Current completion still recreates semantics

`phalcom-lsp/src/completion.rs` includes extensive shallow AST-side receiver/class inference and uses `WorkspaceIndex`.

Part 2 compiler source/advisory products are the replacement. Part 3 deletes the semantic reconstruction while preserving incomplete-text recovery.

## 92.8 Semantic tokens already have a safe fallback seam

`phalcom-lsp/src/semantic_tokens.rs` already has lexer-driven base classification and AST/semantic refinement. Part 3 keeps the lexer fallback and replaces semantic refinement with compiler occurrence products.

---

# 93. Self-review decisions and subtle boundaries

## 93.1 Unified visual type does not mean unified semantic authority

Removing `≈` is a display choice only.

The implementation must never respond by storing advisory facts as formal facts so the renderer can use one code path. The renderer uses one visual syntax over two internal lanes.

## 93.2 “Trusted native” is not runtime privilege

`NativeTrust::{Ordinary, Privileged}` in primitive metadata is not the same concept as formal evidence trust.

Canonical native type metadata imported by the semantic engine is a compiler input. Do not make formal return certainty depend on `trust = privileged`.

## 93.3 Do not infer Unit from side effects

`System.print` is Unit because this specification ratifies that language contract.

A function is not Unit merely because it performs I/O.

## 93.4 Do not infer Unit from `none_value()`

Runtime representation and language type are separate. The native contract audit must respect this.

## 93.5 Do not preserve semantic UX detail at the cost of noise

A fact can carry ten evidence nodes internally while hover shows one useful sentence.

The goal is explainability, not exhaustive proof rendering.

## 93.6 Syntax recovery is allowed

A dangling-dot scanner or incomplete-call parser does not constitute a second semantic analyzer if it only identifies source shape and delegates meaning to compiler products.

## 93.7 Source comments/docs may remain LSP presentation inputs

Phaldoc can remain a source-text presentation concern if the compiler does not own a doc index yet. It must not be used to decide formal type semantics.

## 93.8 Fast-path semantic reconstruction is not allowed

The existence of debounce latency does not justify retaining the old `shallow_receiver_classes` semantic algorithm after compiler source/advisory staged publication exists.

## 93.9 Snapshot-local IDs remain snapshot-local

Part 3 lifecycle work must not attempt to make local binding IDs persistent across arbitrary edits. Cross-revision identity applies where the semantic object itself has cross-revision identity.

## 93.10 Professional presentation must preserve uncertainty somewhere

Removing the visible `≈` does not mean lying about uncertainty. Hover/tooltips and semantic status remain available. The ordinary type label is intentionally concise.

---

# 94. Verification procedure for implementation

Before declaring the implementation complete, run the focused gates in task order, then the broad suite.

Representative sequence:

```bash
cargo test -p phalcom-core --test native_contracts -- --nocapture
cargo test -p phalcom-semantic --test trusted_return_contracts -- --nocapture
cargo test -p phalcom-modules --test workspace_session -- --nocapture
cargo test -p phalcom-semantic --test semantic_single_world -- --nocapture
cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture
cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture
cargo test -p phalcom-lsp --test performance -- --nocapture

cargo check --workspace
cargo test -p phalcom-modules
cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

Then execute and manually inspect §§85–87 forbidden-pattern audits.

Wall-clock benchmarks may be recorded, but they do not replace structural incrementality assertions.

---

# 95. Final interpretation

Parts 1 and 2 make the semantic world correct and singular.

Part 3 makes that world operationally real.

After it lands:

```text
there is one module identity system;
there is one semantic session;
there is one formal checker;
there is one advisory analyzer;
there is one source target/index system;
there is one invalidation/dependency authority;
there is one immutable semantic snapshot per publication;
there is one semantic world consumed by the IDE.
```

The IDE does not expose this architecture as ceremony. It uses the architecture to give ordinary, stable, useful answers:

```text
: String
 -> Unit
x: CellNum
```

and, when the developer asks why, it can answer with the evidence that actually matters.

That is the final single-world takeover contract.
