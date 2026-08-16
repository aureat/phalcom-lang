# Phalcom Modules — Repair & Stabilization Implementation Specification

Remaining correctness repairs, runtime lifecycle hardening, diagnostics, tooling convergence, and regression closure

Status: implementation-ready design specification · Baseline: phalcom-lang @ 5c64793299ab7f29dd7ec4d6a4e76c33dfa14167 · 2026-08-16

## 1. Purpose and status

This specification is the implementation contract for the remaining Phalcom modules repair pass after the first stabilization commit. It consolidates all unresolved findings from the module architecture reviews, the live-source audit, and the subsequent architectural decisions. It is deliberately narrower than the new universe/standard-library architecture: this document repairs the module system so that the architectural migration can land on a sound substrate.

Normative rule: if this document conflicts with the earlier repair plan, this document supersedes it. The separate “Universe, Standard Library, Reflection, and Project-as-Package” specification controls the new builtin-project and runtime-reflection model; this document references that model where required for identity and ownership correctness.

### 1.1 Baseline

| Area | Baseline state | Required direction |
| --- | --- | --- |
| Identity | Resolved project IDs start at 1; reserved 0 exists; synthetic IDs still name-derived | Replace magic reserved/synthetic identity with typed project identity variants |
| Ownership | EntryOwnership enum exists but standalone module compilation still builds pseudo-projects | Ownership becomes authoritative and drives permitted import roots/providers |
| Interface | Three-pass builder exists; import/body collision fixed; duplicate body declarations still overwrite | Single namespace with duplicate rejection across every declaration source |
| Runtime | Topological initialization and register_new exist; materialization/recompilation still mutates existing modules | Program ownership + immutable materialization for already materialized/terminal modules |
| Dispatch | Rest-family routing improved; zero-argument getter/method distinction remains incorrect | Selector kind is authoritative; preserve complete selector shape |
| Errors | ProgramCompileError preserved at interpreter boundary; parse/interface errors still string-erased on some paths | Typed errors retained end-to-end |
| LSP | DocumentModuleMap bidirectional mapping improved | All module identity/resolution delegated to phalcom-modules |
| Static declarations | Linked/static-reference vocabulary exists; declaration blueprints are not yet an authoritative producer-consumer path | Materialize canonical declaration shells and preserve qualified declaration identity end-to-end |
| Builtin roots | Reserved `core` is recognized at resolver boundary but not guaranteed through closed-program discovery/linking | Builtin Projects are first-class provider-backed graph participants; transitional `core` is full-pipeline or rejected early |
| Naming/layout | Logical and physical module/package spellings are not yet a single enforced convention | Logical components are canonical `snake_case`; physical source path components are canonical `kebab-case` with reversible mapping |
| Testing | New regression files exist, but several tests assert weak proxies | Behavioral matrix exercises real filesystem/runtime/process paths and directly observes the named semantic invariant |

## 2. Goals, non-goals, and release gate

### 2.1 Goals

- One semantic identity for every physical/logical module within one resolved program universe.

- No accidental source authority outside the ownership boundary of a project or standalone package.

- No silent namespace overwrite in static interfaces or runtime module registration.

- Deterministic, non-recursive, sticky module initialization with explicit failure states.

- Exact selector-shape preservation for module export dispatch, including Getter versus Method(0).

- Typed diagnostics preserved from parser/interface/resolver through CLI/interpreter/LSP boundaries.

- A single module-resolution implementation shared by compiler, runtime entry selection, CLI, and LSP.

- Canonical linked identity for every cross-module static declaration reference, including qualified superclass references.

- Supported semantic declaration SCCs through predeclaration/materialization of declaration shells, without permitting runtime initialization cycles or inheritance cycles.

- Reversible logical `snake_case` ↔ physical `kebab-case` module/package naming with explicit project namespaces.

- Tests that prove adversarial semantics rather than implementation-local proxies and assert the observable property named by the test.

### 2.2 Non-goals

- No full VM-independent bytecode artifact format in this pass.

- No workspace implementation or dependency solver.

- No complete standard-library migration; only hooks required by the builtin-project model.

- No public reflective export descriptors beyond what is required by the companion architecture spec.

- No performance optimization until the authority and lifecycle model is correct.

### 2.3 Release gate

The module subsystem is considered stabilized only when every P0/P1 item below has an executable regression test, builtin roots survive the complete resolve → discover → link → materialize → initialize pipeline, qualified static declaration references and supported semantic SCCs are operational, the static and dynamic dispatch matrices agree, CLI directory execution is process-tested, and no public API can silently replace a live module identity.

## 3. Normative invariants

| Invariant | Rule |
| --- | --- |
| I-ID-1 | Builtin projects, resolved projects, and synthetic execution contexts occupy disjoint identity variants; they never alias by integer convention. |
| I-ID-2 | A physical source has at most one ModuleId inside a resolver generation. |
| I-OWN-1 | Module discovery authority is derived from EntryOwnership, never inferred by manufacturing a pseudo-project for convenience. |
| I-SRC-1 | Every filesystem resolution is canonicalized and confined to the owning source root before SourceId assignment. |
| I-NS-1 | A module interface has exactly one explicit module namespace; duplicate declarations/import aliases are errors. |
| I-RUN-1 | A ModuleId may be registered once in a runtime program generation unless an explicit reset operation creates a new generation. |
| I-RUN-2 | Initialized and Failed states are sticky; rerunning a compiled/linked program does not rewrite or recompile terminal modules. |
| I-RUN-3 | Initialization executes only in validated topological order and never recursively initializes a dependency. |
| I-DSP-1 | Selector kind, positional count, labels, and rest-family shape participate in dispatch; arity alone is never sufficient. |
| I-ERR-1 | Structured compiler/module errors are not converted to strings before the user-facing formatting boundary. |
| I-LSP-1 | LSP document identity is a view over phalcom-modules identity, not a parallel path-derived model. |
| I-CACHE-1 | Resolver caches are generation-scoped or explicitly invalidated; stale source/interface entries cannot silently survive a generation change. |
| I-BLT-1 | Builtin Project roots are complete graph participants backed by a builtin provider; resolver-only recognition is never considered successful integration. |
| I-PKG-1 | `package.ph` establishes Package identity. `main.ph` may select a default executable entry but never creates Package identity by itself. |
| I-DECL-1 | Every cross-module static declaration reference is resolved to one canonical linked declaration identity and retains that identity through compilation/materialization. |
| I-DECL-2 | Qualified static references never degrade to leaf-name/current-module global lookup. |
| I-DECL-3 | Declaration shells required by another declaration are materialized before dependent semantic realization requires their runtime identity. |
| I-SCC-1 | Semantic declaration SCCs are supported by multi-phase declaration realization; runtime module initialization cycles remain illegal and class inheritance cycles remain independently illegal. |
| I-NAME-1 | Persistent logical namespace/module/package components use canonical `snake_case`; physical module/package source components use canonical `kebab-case`; mapping is deterministic and reversible. |
| I-DUNDER-1 | Dunder reservation is a compiler policy over otherwise valid runtime identifiers/selectors. Unknown dunders are forbidden to user source; specific standardized hooks may opt into narrowly defined user declaration/override roles. |
| I-META-1 | Unit-level metadata uses the contiguous `@!attribute(...)` syntax and attaches to the current source-unit object; unknown metadata is inert unless a standardized attribute definition gives it semantics. |
| I-TEST-1 | A semantic regression test must directly observe the invariant it names; successful execution alone is insufficient when an incorrect implementation could also succeed. |

## 4. Workstream A — semantic identity algebra

The reserved-zero design solved the immediate core collision but still encodes semantic categories as numeric conventions. The new builtin-project architecture requires a stronger identity model and simultaneously closes the synthetic collision problem.

### 4.1 Replace ResolvedProjectId-as-union with ProjectIdentity

```
pub enum ProjectIdentity {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}

pub enum BuiltinProject {
    Universe,
    Std,
}

pub struct ResolvedProjectId(NonZeroU32);
pub struct SyntheticProjectId(u64); // process/session-unique monotonic id

pub struct ModuleId {
    project: ProjectIdentity,
    path: ModulePath,
}
```

Do not expose a general from_raw constructor for reserved/builtin identity. Serialization, if reintroduced later, must serialize the tagged identity, not a bare integer.

### 4.2 Synthetic allocation

- Allocate SyntheticProjectId from a monotonic counter owned by the compile/session universe, not from logical names.

- Logical display names remain metadata only. Two synthetic modules named “inline” must still have distinct semantic identities unless explicitly created as the same synthetic unit.

- Remove ModuleId::synthetic(name) as a name-derived identity constructor. Replace with APIs requiring a SyntheticProjectId allocator/context.

- Remove public VM helpers that overwrite the registry by recreating the same logical synthetic name.

### 4.3 Migration compatibility

During migration, ModuleId::core() may exist only as a temporary compatibility shim returning the builtin Universe root. No new code may pattern-match on project.raw()==0. Delete the shim once the companion universe migration completes.

## 5. Workstream B — ownership and source authority

### 5.1 Make EntryOwnership authoritative

```
pub enum EntryOwnership {
    ProjectOwned { project: ResolvedProjectId },
    StandalonePackageOwned { package_root: CanonicalPath },
    StandaloneModule { file: CanonicalPath },
    Inline { synthetic: SyntheticProjectId },
}
```

The compiler entry classifier produces EntryOwnership once. Every subsequent resolver/source-provider decision derives from it. No branch may create a pseudo-project solely to reuse project resolution logic.

For a directly selected `.ph` file, ownership discovery is ordered and authoritative:

1. Discover an owning persistent Project. If found, classify `ProjectOwned`.
2. Otherwise discover the nearest valid standalone Package hierarchy according to `package.ph` boundaries and classify `StandalonePackageOwned`.
3. Otherwise classify `StandaloneModule`.

A directory is a Package entry only when it has a valid `package.ph`. The presence of `main.ph` alone is never sufficient to create Package identity. `main.ph` is only a default-execution convention for an already identified Package/Project.

### 5.2 Ownership semantics

| Ownership | Permitted local discovery | Project dependencies | Builtin roots |
| --- | --- | --- | --- |
| ProjectOwned | Within project source tree according to module/package rules | Yes | universe, std |
| StandalonePackageOwned | Within the standalone package tree only | No | universe, std |
| StandaloneModule | The file itself only; no sibling discovery | No | universe, std |
| Inline/REPL | No filesystem module discovery unless explicit context attached | No | universe, std |

### 5.3 Restricted providers

```
trait SourceProvider {
    fn locate(&mut self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError>;
}

struct ProjectSourceProvider { /* project-confined */ }
struct StandalonePackageSourceProvider { /* package-root-confined */ }
struct StandaloneModuleSourceProvider { entry: CanonicalPath }
struct BuiltinProjectSourceProvider { builtin: BuiltinProject }
struct InlineSourceProvider { /* no filesystem lookup */ }
```

`StandaloneModuleSourceProvider` must reject every filesystem module lookup other than the entry module. `BuiltinProjectSourceProvider` owns `universe`/`std` source/interface/document identity and never masquerades as an ordinary resolved filesystem Project. Builtin roots are resolved by builtin providers/resolvers, not by widening the filesystem provider or manufacturing display paths such as `<core>` and later treating them as files.

## 6. Workstream C — canonical source identity and confinement

### 6.1 Canonicalization order

1.  Construct the candidate path from a validated logical module/package path.

2.  Canonicalize the candidate filesystem path, resolving symlinks.

3.  Canonicalize the owning root once and retain it in the provider.

4.  Require canonical_candidate.starts_with(canonical_root).

5.  Only after confinement succeeds, allocate/lookup SourceId and bind SourceId ↔ ModuleId.

6.  Reject a second ModuleId for an already-bound SourceId as DuplicateSourceIdentity.

### 6.2 Reverse map contract

```
module_to_source: HashMap<ModuleId, SourceId>
source_to_module: HashMap<SourceId, ModuleId>
```

Insertion is transactional: neither direction may be updated if the other direction conflicts. The reverse map must be consulted in root module, regular module, and package branches.

### 6.3 Required tests

- A package.ph symlink that resolves outside the source root is rejected.

- Two logical paths reaching the same canonical file through symlinks are rejected.

- A legitimate package root canonicalizes and resolves successfully.

- Case-only aliases and Unicode-normalization aliases are diagnosed according to the portability policy in Workstream M.

### 6.4 Logical `snake_case` and physical `kebab-case`

Logical module/package paths and physical source locations deliberately use different canonical spellings.

- Persistent project namespaces are explicit logical identifiers and must be canonical `snake_case`, for example `namespace = "geometry_toolkit"`.
- Logical child package/module components, dependency aliases, and other logical import-root aliases are canonical `snake_case`: `package_a.module_b`.
- Physical package directories and module file stems are canonical `kebab-case`: `package-a/module-b.ph`.
- The resolver maps a physical component to its logical component by canonical `-` → `_` separator conversion after validating the physical spelling; the inverse mapping for source discovery is `_` → `-`.
- Physical `_` in a module/package component and logical `-` in a module/package component are invalid rather than alternate spellings. This preserves a one-to-one mapping.
- The project root directory name is not semantic authority. A directory named `geometry-toolkit` may conventionally accompany `namespace = "geometry_toolkit"`, but only the explicit manifest namespace establishes the Project import root.
- Reserved structural filenames such as `package.ph`, `project.toml`, and any explicitly standardized entry filename are not ordinary path components and are not transformed.

Example:

```text
physical:
    ~/path/to/geometry-toolkit/package-a/module-b.ph

project.toml:
    namespace = "geometry_toolkit"

logical:
    geometry_toolkit.package_a.module_b
```

Validation happens before filesystem probing so two physical spellings can never compete for one logical component. Existing portability normalization/case diagnostics remain applicable to supported non-ASCII filesystem behavior; implementations must not silently accept a second non-canonical spelling.

Required naming diagnostics/tests:

- invalid logical namespace/component using kebab-case or mixed case;
- invalid physical module/package component using snake_case or mixed case;
- canonical round-trip `module_b` ↔ `module-b`;
- project root directory spelling does not override explicit manifest namespace;
- no ambiguous physical alias maps to the same logical component.

## 7. Workstream D — interface namespace and declaration analysis

### 7.1 Three-pass interface build remains, but namespace insertion becomes checked

```
fn declare(namespace: &mut Namespace, name: Symbol, origin: DeclarationOrigin)
    -> Result<(), InterfaceError>
{
    if let Some(previous) = namespace.get(&name) {
        return Err(InterfaceError::DuplicateBinding { name, previous, origin });
    }
    namespace.insert(name, origin);
    Ok(())
}
```

Every explicit module-scope binding uses the same checked insertion helper: class declarations, let/const/pattern declarations, imports, import aliases, and any future declaration form. No direct HashMap::insert is permitted in declaration collection.

### 7.2 Passes

| Pass | Responsibility | Errors |
| --- | --- | --- |
| 1 — declarations | Collect every body declaration into unified namespace and symbol table | Duplicate declaration / duplicate pattern-bound name |
| 2 — imports | Resolve import-introduced local names into same namespace | Import/import and import/declaration collision |
| 3 — exports | Validate exports against completed namespace and imported-module bindings | Unknown export; invalid private/exposure re-export |

### 7.3 Export-before-declaration

Export declarations are declarative interface statements, not source-order effects. Therefore export Foo before class Foo remains valid. Duplicate declarations remain invalid regardless of source order.

## 8. Workstream E — resolver and structured module loading errors

### 8.1 ModuleLoadError retains typed payloads

```
pub enum ModuleLoadError {
    Resolution(ModuleResolutionError),
    Parse { module: ModuleId, error: SyntaxError },
    Interface { module: ModuleId, error: InterfaceError },
    Io { module: Option<ModuleId>, error: io::Error },
}
```

If SyntaxError cannot yet be stored directly because of crate layering, introduce a VM-free ParsedDiagnostic/SyntaxDiagnostic type that retains span, source identity, code, and structured message fields. A String is not an acceptable intermediate representation.

### 8.2 Error propagation

- `ModuleResolver::load_package_surface` and package-exposure traversal return `ModuleLoadError` directly or a typed wrapper preserving its original parse/interface/resolution variant and source range.

- ProgramCompiler inline/interface compilation maps InterfaceError to a dedicated ProgramCompileError::Interface variant.

- CLI and Interpreter format structured errors only at their final presentation boundary.

- LSP converts structured errors to diagnostics without reparsing formatted strings.

## 9. Workstream F — runtime registry, materialization, and program ownership

### 9.1 Registry API

```
impl ModuleRegistry {
    pub fn register_new(&mut self, id: ModuleId, object: ObjRef) -> Result<(), RuntimeError>;
    pub fn get(&self, id: &ModuleId) -> Option<ObjRef>;
    pub(crate) fn reset_generation(&mut self, generation: RuntimeGeneration);
}
```

Delete or make private the unconditional insert API. All normal construction paths, including VM::create_module_with_id, must call register_new. Replacement is not a generic operation.

### 9.2 Runtime generation/program ownership

```
pub struct RuntimeProgramId(u64);

struct ModuleRuntimeRecord {
    object: ObjRef,
    program: RuntimeProgramId,
    plan_fingerprint: ModulePlanFingerprint,
}
```

Before reusing a ModuleId already present in the registry, materialization verifies that the runtime record belongs to the same RuntimeProgramId and has the same immutable plan fingerprint. A separately linked program with coincidentally equal semantic module IDs must not mutate a prior program’s module object.

### 9.3 Idempotent materialization

materialize_program must be a no-op for a module already materialized for the same runtime program and fingerprint. It must not rewrite declarations, linked reads, exports, metadata, source text, or initializer closures.

### 9.4 run_compiled behavior

```
run_linked(program):
    materialize only missing modules
    compile initializer closure only for modules lacking compiled initializer
    initialize Prepared modules in topo order
    skip Initialized modules
    return sticky error for Failed modules without recompilation
```

This closes the current half-idempotence where initialization state is sticky but `run_compiled` still recompiles and rewrites closures before discovering the terminal state.

For the same `RuntimeProgramId` and plan fingerprint, a second run of an `Initialized` or `Failed` module must perform **no parser/compiler work** for that module. Tests must instrument or otherwise observe parse/compile invocation count, not merely object identity.

## 10. Workstream G — initialization state machine

### 10.1 State transitions

```
Allocated -> Prepared -> Initializing -> Initialized
                           |               
                           +-------------> Failed
Prepared --------------------------------> Failed
```

Only initialize_single_module may transition Prepared → Initializing. The topological driver is the only caller during ordinary program initialization.

### 10.2 Dependency checks

- Every runtime dependency must already be Initialized when the current module begins.

- Failed dependency propagates a typed dependency-failure error and marks the current module Failed.

- Prepared or Initializing dependency at this point is an InternalModuleOrderViolation typed error, never a panic.

- No .expect or recursion-based fallback is permitted in initialization.

### 10.3 Partial failure lifecycle

The generation remains valid after a module failure: already initialized dependencies stay initialized; the failing module becomes Failed; downstream modules that require it become Failed when reached. Re-execution of the same runtime program returns the same sticky failure. Starting a new program generation is the explicit mechanism for retrying after source/code changes.

## 11. Workstream H — exact module export dispatch

### 11.1 Selector identity

Dispatch must branch on SignatureKind first. Getter and Method(0) are distinct even though both have zero runtime arguments.

```
match kind {
    SignatureKind::Getter => return exported_value,
    SignatureKind::Method => dispatch_call_protocol(exported_value, slots, labels, args),
    SignatureKind::Rest => dispatch_rest_protocol(...),
    // other existing kinds preserved exactly
}
```

### 11.2 Export forwarding

- Preserve positional count, labeled slots, labels, selector kind, and rest-family information.

- Forward actual argument values to the exported callable; never discard incoming arguments.

- Use identical shape construction in static bytecode send and dynamic send.

- Whole-module exports return a module/package/project object for Getter access and participate in call forwarding only for method selectors.

### 11.3 Shadowing rule

Module/package/project exports remain higher priority than ordinary Module methods for ordinary selector names. The companion spec reserves the dunder protocol namespace under compiler control; specifically non-overridable context/reflection dunders remain a collision-free tooling lane, while separately standardized hook dunders may opt into narrowly defined user override semantics.

## 12. Workstream I — entry selection and CLI

### 12.1 EntrySelection

Context-free EntrySelection::ModuleId must not pretend to compile a module without its linked universe. Either remove the variant from the context-free API or require a LinkedProgram/ProgramCompiler context in the type signature. Erroring with an IO-flavored string is not an acceptable long-term API.

### 12.2 CLI

- Classify path before attempting read_source.

- Directory with project.toml → Project entry.

- Directory with `package.ph` and no project manifest → Package entry.

- Directory containing `main.ph` but no `package.ph` is **not** a Package; `main.ph` is only a default executable entry convention after Package/Project identity is established.

- Regular `.ph` file → Module entry with ownership discovery.

- Inline -e/text → Inline entry.

- Add real subprocess tests asserting exit status and observable output for project/package directory invocation.

## 13. Workstream J — project metadata correctness

### 13.1 Display name versus namespace

```
struct ValidatedProjectManifest {
    display_name: String,      // exact validated user-facing project name
    namespace: ModuleComponent,
    // ...
}
```

ResolvedProject.name must receive display_name, never namespace. The companion architecture spec makes namespace mandatory for persistent projects; this repair must preserve both fields independently in preparation for that rule.

### 13.2 Manifest strictness

- Keep deny_unknown_fields on project/dependency records.

- Reject mixed path + package/version dependency declarations.

- Provide dedicated validation errors for missing mandatory namespace after the migration flag is enabled.

- Cycle diagnostics report the exact repeated project and the minimal cycle path.

### 13.3 Unit-level metadata syntax and authority

The canonical unit-level attribute syntax is contiguous:

```phalcom
@!documentation("...")
@!some_metadata(...)
```

`@!` and the attribute name form one source construct; no whitespace, newline, or comment trivia may appear between `!` and the attribute identifier. `@! documentation(...)`, `@!\ndocumentation(...)`, and `@!/*...*/documentation(...)` are invalid. Do not introduce `@module.*`, `@package.*`, or `@project.*` as competing targeting syntax.

Conceptual grammar:

```text
unit_attribute := "@!" IDENT attribute_arguments?
```

A unit attribute is a declarative top-level unit-header item. It is **not** an executable statement and is not an ordinary attribute waiting to attach to the next class/method/declaration. Reordering unit metadata entries does not change module initialization behavior.

Semantics:

- `@!attribute(...)` targets the **current source-unit runtime/static object**.
- In an ordinary module source it targets that `Module`.
- In standalone/nested `package.ph` it targets that `Package`.
- In a Project root `package.ph` it targets the `Project` object itself; because `Project : Package : Module`, no duplicate facet object is created.
- Metadata is inert by default. An unknown metadata attribute is retained as data and acquires no compiler/runtime behavior merely because of its spelling.
- Standardized semantic attributes must be registered/defined explicitly with allowed unit kinds and semantics. Remove hard-coded string checks such as “package-only because the name equals X” unless X is a formally specified attribute.
- If a standardized attribute is valid for `Package`, applicability to `Project` follows the declared unit-kind hierarchy where intended; attachment still occurs once to the Project object.

Parser/compiler diagnostics must distinguish malformed `@!` syntax, an unknown-but-valid inert metadata attribute, and a known semantic attribute used on an invalid unit kind.

## 14. Workstream K — static declaration planning, semantic SCC realization, and artifact truthfulness

### 14.1 Canonical static declaration identity

The interim linked program must contain semantically complete declaration plans for every declaration whose identity is needed across module boundaries. This is required even before Phalcom has a fully VM-independent executable artifact.

A qualified static reference such as:

```phalcom
import .base

class Child is base.Shape {
}
```

must follow one authority chain:

```text
source spelling `base.Shape`
    → linker resolves canonical SymbolId / linked declaration identity
    → declaration plan stores that identity
    → materializer resolves/creates the corresponding runtime declaration shell
    → compiler/runtime uses that shell identity
```

It must never degrade to `GetGlobal("Shape")`, current-module leaf-name lookup, import-alias string recovery, or filesystem path reconstruction.

A representative plan shape is:

```rust
pub struct ClassBlueprint {
    pub symbol: SymbolId,
    pub defining_module: ModuleId,
    pub name: Symbol,
    pub superclass: Option<LinkedClassRef>,
    // fields/method metadata required before body realization
}
```

Exact fields are implementation-defined; canonical identity preservation is not.

### 14.2 Declaration-shell materialization

`ModuleMaterializationPlan::declarations` (or its renamed equivalent) is an active producer-consumer path, not empty scaffolding.

Implementation phases for declarations:

1. Collect and link declaration identities.
2. Build declaration blueprints.
3. Allocate/materialize declaration shells for the relevant semantic component.
4. Resolve static edges (superclass/type/other supported declaration references) against those shells.
5. Validate edge-specific invariants.
6. Compile/install declaration bodies and module initializer behavior.
7. Execute module initialization only according to the separate runtime dependency DAG.

Cross-module class reopens/attachments must resolve an existing canonical class identity rather than allocate a same-named duplicate.

### 14.3 Semantic declaration SCCs — supported now

Semantic SCCs are a supported Modules v1 feature and must not be deferred.

For each semantic SCC:

```text
Phase A — predeclare/materialize every declaration shell in the SCC
Phase B — resolve canonical static references among those shells
Phase C — validate constraints and realize bodies
```

This permits legal mutually referential declarations without introducing effectful module initialization order.

Important separations:

- A semantic-reference SCC is legal when all participating declaration kinds/edges permit it.
- The module **runtime initialization graph remains acyclic** and is still executed topologically.
- The class superclass/inheritance graph remains independently acyclic. A semantic SCC does not legalize `A is B` / `B is A`.
- Edge-specific invalid cycles produce typed semantic diagnostics identifying the declaration cycle, not a generic module-runtime-cycle error.

### 14.4 Compiled-plan/artifact truthfulness

The current source-retaining program plan is not yet a VM-independent compiled artifact. Rename it now rather than preserving misleading abstractions. Renaming does **not** permit empty declaration scaffolding: the interim linked plan must still carry the declaration identities/blueprints required by currently supported module semantics.

| Current name | Interim truthful name | Future reserved meaning |
| --- | --- | --- |
| CompiledProgram | LinkedProgramPlan or LinkedSourceProgram | A genuinely compiled executable program artifact |
| ModuleArtifact | ModuleMaterializationPlan | VM-independent per-module executable artifact |

No type described as VM-independent may contain ObjRef. If an ObjRef is required, the type belongs to runtime materialization, not compilation output.

## 15. Workstream L — LSP convergence

### 15.1 Single authority

- Replace local/path-derived LSP ModuleId with phalcom_modules::ModuleId.

- DocumentModuleMap remains a URI/document cache, not an identity authority.

- All import target calculation and package/project ownership uses `ModuleResolver`/`ProjectUniverse`.

- Project-backed, standalone-package, standalone-module, and builtin documents enter semantic analysis through an authoritative resolved-document record. The semantic engine consumes resolved module identities and import edges; it does not reconstruct project/dependency imports from URI or filesystem spelling.

A representative boundary is:

```rust
struct ResolvedDocument {
    uri: Url,
    source: SourceId,
    module: ModuleId,
    program: ParsedProgram,
    imports: Vec<ResolvedImportEdge>,
    generation: SourceGeneration,
}
```

Exact field placement is implementation-defined. The invariant is that once this boundary has been crossed, `SemanticEngine` never runs a second project/dependency resolver.

- Absolute own-project roots, dependency aliases, exposure rules, and builtin roots must resolve identically in compiler and LSP.

- URI heuristics are permitted only for genuinely unprojected/unsaved editor buffers. Such ephemeral documents may receive local syntax/semantic service but may not invent Project/dependency import edges.

- Remove linear reverse scans once canonical bidirectional maps are available.

- Generation changes invalidate semantic caches keyed by module/source identity.

### 15.2 Builtin virtual document identity

Builtin sources use stable virtual document URIs derived from logical `ModuleId`, not filesystem display paths:

```text
phalcom://universe/
phalcom://universe/reflection/selector
phalcom://std/json
```

Rules:

- URI host identifies the builtin Project (`universe`, `std`).
- URI path uses logical `snake_case` module/package components, never physical `kebab-case` spelling and never a `.ph` suffix.
- URI generation is deterministic from builtin `ModuleId` and is independent of where the toolchain is installed.
- Builtin virtual documents are read-only from the editor/LSP perspective.
- If embedded builtin source has a backing toolchain file, that physical path is provenance/source-map metadata; it does not replace the canonical `phalcom://` document URI.
- `Url::from_file_path` must never be used on synthetic/builtin display strings.

### 15.3 Migration sequence

1.  Introduce adapters from LSP URI → canonical SourceId using the shared source provider.

2.  Re-key semantic documents by shared ModuleId.

3.  Route import diagnostics/navigation through ModuleResolver.

4.  Delete legacy URI heuristics only after parity tests pass.

5.  Add tests proving compiler and LSP resolve the same import graph for the same fixture.

## 16. Workstream M — cache generations and portability diagnostics

### 16.1 Resolver generations

```
pub struct ResolverGeneration(u64);

struct SourceCacheEntry<T> {
    generation: ResolverGeneration,
    value: T,
}
```

Prefer immutable resolver snapshots for a single compile/check operation. If a long-lived resolver is retained by the LSP, changing filesystem/project configuration increments generation and invalidates or namespaces all path/interface/source mappings.

### 16.2 Portability policy

- Detect module/package components that collide under Unicode normalization expected on supported filesystems.

- Detect case-fold collisions when two paths would be indistinguishable on a case-insensitive filesystem.

- Report these as portability diagnostics even when the current filesystem can distinguish them.

- Do not silently normalize logical source spelling; diagnostics should point at both conflicting declarations/files.

## 17. Required regression matrix

| Test | Behavior |
| --- | --- |
| ID-01 | First resolved project cannot alias universe/std builtin identities |
| ID-02 | Two same-name synthetic modules get distinct ProjectIdentity values |
| ID-03 | No public raw constructor can manufacture builtin identity |
| OWN-01 | Standalone module cannot import sibling file |
| OWN-02 | Standalone package child can import within package |
| OWN-03 | Project-owned entry sees project dependencies |
| OWN-04 | Standalone contexts see universe/std but not project deps |
| PKG-01 | Directory with `main.ph` but no `package.ph` is not classified as Package |
| PKG-02 | Directory with `package.ph` has Package identity independently of whether `main.ph` exists |
| SRC-01 | Symlinked root package escaping source root rejected |
| SRC-02 | Two logical aliases to same canonical source rejected |
| SRC-03 | Root package valid confinement succeeds |
| NAME-01 | Explicit `namespace = "geometry_toolkit"` and dependency/import-root aliases validate as logical snake_case; kebab/mixed-case logical names are rejected |
| NAME-02 | `package-a/module-b.ph` maps to logical `package_a.module_b`; physical snake_case/mixed-case component is rejected |
| NAME-03 | Logical/physical mapping round-trips without aliases; project root directory spelling never overrides manifest namespace |
| NS-01 | Duplicate class declaration rejected |
| NS-02 | Duplicate let/pattern declaration rejected |
| NS-03 | Import/declaration collision rejected |
| NS-04 | Export before declaration succeeds |
| NS-05 | Cross-kind duplicate (`class Foo` plus `let Foo`) is rejected at interface construction |
| DECL-01 | Qualified superclass `base.Shape` resolves to canonical remote declaration identity |
| DECL-02 | Same leaf declaration name in two modules cannot confuse a qualified static reference |
| DECL-03 | Required declaration shells exist before dependent semantic realization |
| DECL-04 | Qualified static reference is never lowered/recovered as leaf-only current-module global lookup |
| DECL-05 | Cross-module inheritance executes end-to-end against the intended class identity |
| SCC-01 | Legal semantic declaration SCC is predeclared/resolved/realized successfully |
| SCC-02 | Inheritance cycle inside/alongside semantic SCC is rejected with typed inheritance-cycle diagnostic |
| SCC-03 | Semantic SCC does not create or legalize a runtime module initialization cycle |
| ERR-01 | Syntax error span survives ModuleLoadError and CLI formatting |
| ERR-02 | InterfaceError survives ProgramCompileError/Interpreter boundary |
| ERR-03 | Parse/interface error encountered during package exposure traversal retains original typed diagnostic and source range |
| RUN-01 | Whole-module re-export links and executes |
| RUN-02 | Second materialize of same runtime program is no-op |
| RUN-03 | Second run does not recompile initialized module closure |
| RUN-04 | Failed module remains sticky without recompilation |
| RUN-05 | Dependency failure propagates without panic |
| RUN-06 | Invalid internal order returns typed error, never panic |
| RUN-07 | Second program cannot mutate registry entry owned by first program |
| RUN-08 | `register_new` rejects duplicate identity |
| BUILTIN-01 | `universe` import completes resolve → discover → link → materialize → initialize → execute |
| BUILTIN-02 | Transitional `core` is either a complete alias of universe through the full pipeline or is rejected immediately with the deliberate removal diagnostic |
| BUILTIN-03 | Transitional `core.foo` never collapses to the `core`/universe root identity |
| BUILTIN-04 | Builtin source/interface loading uses `BuiltinProjectSourceProvider`, never resolved-project-zero/filesystem fallback |
| DSP-01 | Getter export returns value |
| DSP-02 | Method(0) export invokes call protocol rather than getter path |
| DSP-03 | Positional arguments preserved |
| DSP-04 | Labeled arguments preserved |
| DSP-05 | Rest-family static and dynamic behavior agree |
| DSP-06 | Export shadows ordinary Module name but dunder reflection remains reachable |
| CLI-01 | Project directory executes through CLI process |
| CLI-02 | Standalone package directory executes through CLI process |
| LSP-01 | Relative import resolves to identical compiler/LSP `ModuleId` and edge |
| LSP-02 | Own-project absolute namespace import has compiler/LSP parity |
| LSP-03 | Dependency-alias absolute import has compiler/LSP parity |
| LSP-04 | Exposure/private rejection agrees between compiler and LSP |
| LSP-05 | Builtin universe module maps to canonical `phalcom://universe/...` URI and identical semantic identity |
| LSP-06 | Builtin std module maps to canonical `phalcom://std/...` URI and identical semantic identity |
| META-01 | `@!documentation(...)` parses as declarative unit-header metadata; whitespace/newline/comment separation after `@!` is rejected |
| META-02 | Project-root `@!` metadata attaches once to the Project object; standalone package/module attach to their current unit object |
| META-03 | Unknown valid `@!` metadata remains inert; no magic string-name semantics |
| DUNDER-01 | Unknown `__name__`-shaped user declaration is rejected by compiler policy while runtime symbol/selector machinery can represent the name |
| DUNDER-02 | A standardized overridable dunder hook is accepted only in its compiler-authorized declaration role; non-overridable reflection dunders remain forbidden |
| TEST-Q-01 | Representative dispatch/init/source tests assert concrete results/counters/identities, not only `is_ok()` |
| CACHE-01 | Generation change invalidates stale source/interface mapping |
| PORT-01 | Case-fold collision diagnostic |
| PORT-02 | Unicode normalization collision diagnostic |
| GC-01 | Linked module reads and whole-module exports remain rooted while program is live |
| GC-02 | Reset/new generation permits old runtime graph to become collectable when otherwise unreachable |

### 17.1 Required fixture topology and integration layers

Tests should share small purpose-built fixtures rather than reconstructing semantic layouts ad hoc in each test. At minimum create these fixture families:

```text
fixtures/modules/
  standalone-sibling-rejection/
    app.ph
    helper.ph

  standalone-package/
    package.ph
    main.ph
    tools/
      package.ph
      run.ph

  main-without-package/
    main.ph

  naming/
    project.toml                 # namespace = "geometry_toolkit"
    src/
      package.ph
      package-a/
        package.ph
        module-b.ph

  qualified-superclass/
    project.toml
    src/
      package.ph
      base.ph
      child.ph
      other-base.ph              # same leaf class name where useful

  semantic-scc/
    project.toml
    src/
      package.ph
      a.ph
      b.ph

  builtin-full-pipeline/
    project.toml
    src/
      package.ph
      main.ph                    # imports universe and observes a builtin export

  metadata-dunder/
    project.toml
    src/
      package.ph
      metadata.ph
      dunder-policy.ph
```

Integration layers:

1. **Resolver/interface tests** assert exact `ModuleId`, `SourceId`, ownership, error variant, and canonical logical/physical mapping.
2. **ProgramCompiler/link tests** assert the closed graph, linked declaration identities, semantic SCCs, builtin nodes, and exposure results.
3. **VM/runtime tests** execute the linked plan and assert concrete values, initializer counters, sticky failure identity, export dispatch, class identity, and GC rooting.
4. **CLI process tests** invoke the actual executable on Project/Package directories and assert exit code + stdout/stderr.
5. **LSP integration tests** run the production semantic-engine ingestion path, not a helper-only resolver path, and assert `URI ↔ SourceId ↔ ModuleId` plus import-edge parity.

The `builtin-full-pipeline` integration test must explicitly fail if it only proves `resolve_import(universe)`: it must reach execution and observe a known builtin export. During transitional compatibility, repeat the same closed-program route for `core` if the alias remains enabled.

For semantic SCC fixtures, use at least one **legal mutual semantic reference** that is not an inheritance cycle, plus a separate explicit inheritance-cycle negative fixture. This proves SCC support without weakening superclass acyclicity.

## 18. Implementation order

| Phase | Changes | Exit condition |
| --- | --- | --- |
| 0 — Freeze | Add failing regression fixtures for all known P0 issues | Tests reproduce every current defect before implementation |
| 1 — Identity | ProjectIdentity variants, synthetic allocator, remove raw reserved construction | No identity-category aliasing |
| 2 — Ownership/source | Authoritative EntryOwnership, restricted providers, confinement, reverse map | Standalone/project boundaries enforced |
| 3 — Interfaces/errors | Checked unified namespace; typed load/compile errors | No silent overwrite or string-erasure seam |
| 4 — Builtin/static declaration authority | Builtin provider/full-pipeline root handling; declaration blueprints; qualified static references; semantic SCC realization | Builtins are graph-complete and cross-module declaration identity is operational |
| 5 — Runtime lifecycle | Registry API, RuntimeProgramId, idempotent materialization/run, topo state machine | Repeated run and failure semantics stable |
| 6 — Dispatch | Getter/Method(0), labels/rest parity, argument forwarding | Static/dynamic matrix green |
| 7 — Metadata/naming/CLI | `@!` unit metadata; snake↔kebab validation; name/namespace; entry APIs/process tests | Public source/entry surfaces coherent |
| 8 — LSP/cache | Resolved-document ingestion, virtual builtin URIs, generation invalidation, portability checks | Compiler/LSP parity green |
| 9 — Cleanup | Rename pseudo-artifacts; delete compatibility APIs and stale comments | No misleading or duplicate authority remains |

## 19. Acceptance criteria

- All P0/P1 regressions in §17 pass under cargo test for affected crates and an end-to-end CLI process suite.

- No public registry API silently overwrites ModuleId.

- No name-derived synthetic identity remains.

- No standalone module is represented as a permissive synthetic project.

- No explicit namespace declaration path performs unchecked overwrite.

- No module parse/interface error is reduced to String before final formatting.

- Getter and Method(0) are demonstrably distinct in module export dispatch.

- Repeated execution of an initialized/failed linked program is observationally idempotent with respect to module object, closure, state, and exports.

- Compiler and LSP share module identity/resolution on relative, own-project absolute, dependency-alias, exposure, and builtin fixtures.

- Builtin `universe` survives the complete closed-program pipeline through a provider-backed graph node; no reserved root is “resolver-only.”

- Qualified cross-module declaration references preserve canonical identity through declaration blueprint/materialization; no leaf-name fallback remains.

- Legal semantic declaration SCCs are realized by predeclaring/materializing shells, while runtime initialization cycles and inheritance cycles remain rejected.

- `package.ph` establishes Package identity; `main.ph` alone never does.

- Logical module/package/namespace components and physical source components obey the canonical snake_case ↔ kebab-case mapping without aliases.

- `@!attribute(...)` is the only unit-level metadata syntax in this design and unknown metadata remains inert.

- Dunder restrictions are compiler-owned: unknown/reserved dunders are rejected, while only specifically standardized protocol hooks may be user-declared/overridden in authorized roles.

- Builtin LSP documents use stable `phalcom://universe/...` / `phalcom://std/...` URIs.

- The compatibility core identity is either removed or reduced to a temporary full-pipeline alias for builtin Universe pending completion of the companion migration.

## 20. Files expected to change

| Area | Likely paths |
| --- | --- |
| Identity/project universe | phalcom-modules/src/identity.rs; project.rs; manifest.rs |
| Sources/ownership | phalcom-modules/src/source.rs; compile entry classification in phalcom-core/src/modules/compile.rs |
| Interface/linking | phalcom-modules/src/interface.rs; linker graph types; static-reference/SymbolId resolution; semantic graph/SCC realization |
| Errors/resolver | phalcom-modules/src/error.rs; resolver.rs; phalcom-core/src/error.rs |
| Runtime | phalcom-core/src/modules/{registry,materialize,initialize,compile,artifact}.rs; class/declaration blueprint production and shell materialization |
| Dispatch | phalcom-core/src/vm/send.rs and selector/signature helpers |
| CLI | phalcom-core/bin/phalcom/cli.rs |
| LSP | phalcom-lsp/src/semantic/* plus shared resolver integration, resolved-document ingestion, `phalcom://` builtin document mapping |
| Tests | phalcom-modules/tests/*; phalcom-core/tests/*; filesystem naming/package fixtures; semantic-SCC/cross-module-class fixtures; CLI process tests; LSP integration/virtual-document tests |
