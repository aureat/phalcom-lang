# Phalcom Live Semantic Intelligence and VS Code Integration
## Professional Implementation Specification

**Repository:** `aureat/phalcom-lang`  
**Design baseline:** `5b6d67be93d6167558931a5c5dae3ae69959c9c4`  
**Status:** Proposed implementation specification  
**Primary components:** `phalcom-lsp`, `tools/vsphalcom`  
**Related components:** `phalcom-ast`, `phalcom-core/core/core.ph`, native primitive registration surface

---

# 1. Purpose

This specification defines the next language-intelligence architecture for Phalcom: a live, VM-free semantic engine that infers runtime classes/value shapes from source and uses those facts for autocomplete, inlay type hints, hover, and cross-file navigation.

The design deliberately does **not** require Phalcom to have a formal static type system. It also does not invent one inside the editor. The server computes advisory facts about the runtime objects that expressions are likely or guaranteed to produce under the currently visible source program.

The implementation replaces two limitations in the existing LSP:

- `ConstructResolver`, which is a narrow local class-name heuristic;
- `CoreTable`, which embeds a generated snapshot of core/native selectors.

The end state must feel live: adding a method to a class in an unsaved buffer changes completion immediately; changing a local binding changes its inlay hint and member surface; method-result and call-site evidence can flow through the workspace; core Phalcom source uses the same indexing path as user source; and the VS Code extension consumes all of this through standard LSP.

---

# 2. Normative language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

A requirement marked “future-compatible” does not need to expose user-visible behavior in the first implementation, but the data model MUST not preclude it.

---

# 3. Goals

The implementation MUST provide:

1. best-effort inference of runtime class/value shape for local bindings and expressions;
2. standard LSP type inlay hints derived from that inference;
3. member completion for arbitrary recoverable receiver expressions, including chained sends;
4. method-return inference from Phalcom method bodies;
5. call-site parameter inference when dispatch can be resolved sufficiently;
6. field inference from class-local assignments;
7. module-aware and cross-file semantic identities;
8. live core-source indexing without `core-table.json`;
9. one canonical source of truth for native primitive selector surfaces;
10. bounded, dependency-aware recomputation on edits;
11. a thin VS Code client with configuration synchronization, reliable server discovery, restart/logging commands, and real E2E tests.

The implementation SHOULD also improve hover by showing inferred value knowledge and provenance where useful.

---

# 4. Non-goals

The first implementation MUST NOT:

- reject a program because an inferred runtime class differs from another inferred class;
- emit red diagnostics solely from speculative inference;
- execute user code or instantiate a Phalcom VM to answer completion;
- assume a capitalized receiver call returns an instance of the receiver class;
- infer opaque native return values without source or canonical metadata;
- create a second parser or grammar in TypeScript;
- collapse module-qualified class identity back to a global bare-name map;
- require public type-annotation syntax merely to display inlay hints.

Formal typing may be added to Phalcom later. This semantic database is designed so explicit annotations can become authoritative facts at that time.

---

# 5. Architectural decision: infer runtime value knowledge, not static types

The core data type SHOULD be named `ValueKnowledge` or `ValueShape`, not `Type`.

Recommended initial representation:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ValueShape {
    Unknown,
    Instance(ClassId),
    ClassObject(ClassId),
    Module(ModuleId),
    Tuple(Vec<ValueShape>),
    Record(Vec<(LabelId, ValueShape)>),
    List(Box<ValueShape>),
    Set(Box<ValueShape>),
    Map {
        key: Box<ValueShape>,
        value: Box<ValueShape>,
    },
    Range(Box<ValueShape>),
    Callable(CallableId),
    Union(SmallVec<[ValueShape; 4]>),
}

pub struct InferredValue {
    pub shape: ValueShape,
    pub confidence: Confidence,
    pub provenance: SmallVec<[FactOrigin; 2]>,
}

pub enum Confidence {
    Exact,
    Flow,
    Interprocedural,
    Heuristic,
}
```

`Unknown` means “the analyzer does not know.” It MUST NOT be treated as “all classes are known possibilities,” and it MUST NOT be rendered as a fake `Any` type unless the language later defines such a type.

`Union` MUST be bounded. The initial implementation SHOULD cap distinct alternatives at `8`. Adding a ninth incompatible alternative MUST widen to `Unknown` rather than allowing unbounded workspace growth.

The representation MAY preserve structured collection element knowledge internally even before Phalcom has generic type syntax. User-facing rendering can remain conservative.

---

# 6. Semantic identity

## 6.1 Module identity

Introduce:

```rust
pub struct ModuleId(InternedUri);

pub struct ClassId {
    pub module: ModuleId,
    pub name: Symbol,
}

pub struct CallableId {
    pub owner: ClassId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

A class MUST never be identified by bare name alone.

This follows the correctness direction already present in `WorkspaceIndex`, which keys classes by `(URI, name)`.

**Current reference:** [`index.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/index.rs#L150-L350)

## 6.2 Core module identity

Core classes MUST receive stable semantic `ModuleId`s just like workspace modules. There MUST NOT be an unrelated “builtin class name” namespace that completion handles through a separate table.

---

# 7. New `SemanticDb`

Add an internal semantic subsystem under `phalcom-lsp/src/semantic/` for the first implementation. Do not create a new workspace crate until another consumer actually needs it; premature extraction would increase API surface without increasing correctness.

Recommended layout:

```text
phalcom-lsp/src/semantic/
  mod.rs
  ids.rs
  module_graph.rs
  surface.rs
  facts.rs
  flow.rs
  callable.rs
  infer.rs
  query.rs
  core_source.rs
  invalidation.rs
```

`Backend` becomes:

```rust
pub struct Backend {
    client: Client,
    documents: DocumentStore,
    semantic: SemanticDb,
    config: RwLock<ServerConfig>,
}
```

`WorkspaceIndex` SHOULD be migrated into or wrapped by `SemanticDb`. Selector-definition/reference indexing remains useful, but completion MUST stop treating `WorkspaceIndex` as the semantic authority.

---

# 8. Semantic snapshots and revisions

Every source file MUST carry a monotonically increasing semantic revision.

```rust
pub struct FileRevision(u64);

pub struct FileSemanticSnapshot {
    pub revision: FileRevision,
    pub module: ModuleId,
    pub surface: ModuleSurface,
    pub local_facts: LocalFacts,
    pub dependencies: DependencySet,
}
```

A `didChange` MUST follow this order:

```text
1. update full source text
2. parse/recover AST
3. increment file revision
4. replace this file's syntactic/surface contribution
5. update import edges
6. extract local facts + callable bodies
7. invalidate reverse semantic dependents
8. recompute the bounded affected work queue
9. atomically publish the new semantic generation
10. publish diagnostics / request inlay refresh as necessary
```

Request handlers MUST read a coherent semantic generation. They MUST NOT observe half-updated maps where a call summary belongs to a newer file revision than its class surface.

The implementation MAY continue full-file parsing initially. Semantic invalidation, however, MUST be dependency-based from the first interprocedural version so future incremental parsing is an optimization rather than an architectural rewrite.

---

# 9. Module graph

The current LSP ignores imports. That must change before cross-file inference.

`phalcom_ast::ast::ImportStatement` already records path, binding, and source range. The semantic layer MUST construct a `ModuleGraph` during workspace/core scanning and update the importing file's edges on each edit.

The graph MUST record:

```rust
pub struct ImportEdge {
    pub from: ModuleId,
    pub binding: Symbol,
    pub target: Option<ModuleId>,
    pub source_range: SourceRange,
}
```

Path resolution MUST use the same rules as the compiler. Shared filesystem/path normalization logic SHOULD live in a VM-free helper (`phalcom-common` is appropriate if the abstraction is compiler-neutral). The LSP MUST NOT duplicate path semantics in an editor-only implementation if the compiler already has canonical rules.

Unresolved imports SHOULD remain represented as edges with `target = None`, so a later file creation can repair the graph without rebuilding unrelated semantic state.

The server MUST handle `didChangeWorkspaceFolders` and watched `.ph` files so module surfaces remain current when closed files are created, changed, or deleted.

---

# 10. Class and member surface model

Every class surface MUST include:

```rust
pub struct ClassSurface {
    pub id: ClassId,
    pub superclass: Option<ClassRef>,
    pub members: BTreeMap<Selector, MemberSurface>,
    pub fields: BTreeMap<FieldId, FieldSurface>,
}

pub struct MemberSurface {
    pub callable: CallableId,
    pub kind: MemberKind,
    pub visibility: MemberVisibility,
    pub source_range: SourceRange,
    pub name_range: SourceRange,
    pub params: Vec<ParamSurface>,
}
```

This model subsumes the member information currently split across `WorkspaceIndex` and `CoreTable`.

Selector identity MUST remain canonical ADR-0012 comma-form. No inference work is permission to introduce bare-selector aliases.

Visibility filtering already present in completion MUST be preserved.

---

# 11. Live core and native surfaces — removal of `core-table.json`

## 11.1 Source-authored core classes

`phalcom-core/core/core.ph` MUST be parsed directly into the semantic database.

For development workspaces, an open or workspace copy of `core.ph` MUST take precedence so edits are reflected live.

For packaged installations, the language server SHOULD discover core source through this precedence:

```text
1. explicit `phalcom.lsp.sysrootPath`
2. core source shipped adjacent to/bundled with the installed language server
3. development-repository discovery
```

An embedded source string is acceptable as a final installation fallback because it is source-of-truth code, not a generated selector table. A workspace/open-buffer copy MUST still override it in development.

## 11.2 Native primitive surface

The current generator text-scans `Universe::install_primitives`.

**Current references:** [`gen-core-table`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/bin/gen-core-table/main.rs#L1-L260), [`primitives.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/src/universe/primitives.rs#L1-L220)

The final architecture MUST have one canonical, structured native declaration consumed or validated by both runtime registration and editor semantics. A generated JSON bridge MUST NOT remain.

### Preferred design: source declarations

Prefer declaring native selectors in core Phalcom source using the existing/extended native member mechanism. If today's `@native` facility cannot express a bodyless native selector surface, extend it minimally so it can.

Conceptual example, subject to the language's actual attribute grammar:

```phalcom
class String {
  @native
  _$byteCount

  @native
  _$byteAt(_ index)
}
```

The runtime registration path then MUST validate that its installed selector/kind/side agrees with the declared surface. The LSP simply indexes the source declaration.

This has three advantages:

- method existence and selector spelling are live source;
- runtime and tooling can detect drift;
- no Rust-source parser and no generated editor artifact are needed.

### Transitional alternative: shared VM-free descriptor crate

If source declarations cannot land in the same unit, introduce a small VM-free native-surface crate containing structured descriptors that are consumed by both runtime registration and `phalcom-lsp`. This is acceptable only as a transitional implementation. It MUST be hand-authored canonical data, not output from a generator, and the runtime MUST consume the same descriptors so drift is impossible.

### Native return knowledge

Selector presence does not imply return shape. If a native implementation is opaque and has no source-level semantic contract, its return MUST be `Unknown`.

Do not guess that `String#size` returns `Int` merely because the name looks obvious. Where a stable return contract is required for editor quality, add that contract to the canonical native declaration/descriptor rather than creating an LSP-only lookup table.

---

# 12. Inference rules

Inference proceeds from strongest, local evidence outward. Rules MUST be conservative with dynamic dispatch.

## 12.1 Exact syntax facts

The following expressions produce exact initial shapes:

| Expression | Shape |
|---|---|
| integer literal | instance of `Int` |
| float literal | instance of `Float` |
| string literal | instance of `String` |
| boolean literal | instance of `Bool` |
| symbol literal | instance of `Symbol` |
| tuple literal | `Tuple` with positional/labeled element facts |
| record literal | `Record` with field value facts |
| list literal | `List<join(elements)>` internally |
| set literal | `Set<join(elements)>` internally |
| map literal | `Map<join(keys), join(values)>` internally |
| range expression | `Range<join(bounds)>` where defensible |
| block literal | callable/block shape |
| `self` | exact current class instance |
| `super` | current superclass receiver semantics |
| resolved class declaration name | class object |

Operators MUST NOT receive hard-coded arithmetic return types simply because they are spelled `+`, `-`, etc. In Phalcom they are ordinary sends. Their results flow through normal dispatch/callable-summary inference.

## 12.2 Bindings and reassignment

For:

```phalcom
let p = Point.new()
```

the binding fact is the initializer fact.

A later assignment in the same control-flow path replaces or joins the previous fact according to flow semantics. Mutable rebinding MUST never leave a stale exact inlay hint.

`const` does not inherently create a stronger class fact than `let`; it creates a stability guarantee about rebinding, which may allow caching/narrowing.

## 12.3 Constructor recognition

Delete `class_of_construct` as the authority.

A call may infer an instance of `ClassId` only when one of these is true:

- the resolved member is a declared constructor/factory whose semantic contract is “instance of owner”;
- the language's built-in `new` semantics guarantees owner-instance creation;
- an analyzed source method summary resolves to that class.

A generic class-side method call MUST otherwise use its callable return summary or remain `Unknown`.

## 12.4 Destructuring

Tuple/list destructuring SHOULD project element facts where the initializer has structured knowledge.

```phalcom
const (x, y) = (Point.new(), "name")
```

can infer `x ≈ Point`, `y ≈ String`.

An unknown or incompatible initializer MUST degrade the affected bindings to `Unknown`, not manufacture an error diagnostic.

## 12.5 `for` bindings

If an iterable has a known element shape, the loop binding receives that shape.

When element shape is unknown, the loop variable remains `Unknown`.

## 12.6 Fields

Collect class-scoped field writes from constructor/member bodies:

```phalcom
_client = HttpClient.new()
```

A field fact is keyed by `(ClassId, FieldKind, field name)`. Writes across reachable methods join. Constructor writes have strong provenance but MUST not be assumed to dominate all possible instances unless language construction rules guarantee that constructor path.

A field read (`_client`, implementation field, or equivalent AST shape) queries this fact.

## 12.7 Callable return summaries

Each source method/getter/setter/index member gets a summary:

```rust
pub struct CallableSummary {
    pub params: Vec<InferredValue>,
    pub returns: InferredValue,
    pub effects: SummaryEffects,
    pub dependencies: SmallVec<[CallableId; 4]>,
    pub revision: SemanticGeneration,
}
```

Return inference MUST follow Phalcom's actual method/block value semantics, including explicit `return` and the language-defined implicit final-result path. Do not approximate this with “last AST expression” unless that is exactly the compiler rule.

A result such as:

```phalcom
makePoint() { Point.new() }
```

can therefore feed completion on:

```phalcom
makePoint().
```

## 12.8 Parameter inference from call sites

For a resolved call target, argument facts flow into corresponding parameters.

```phalcom
draw(_ shape) { shape. }
draw(Circle.new())
```

can infer `shape ≈ Circle` inside `draw` when the call target is unambiguous.

Across several calls, parameter facts join. A `Circle` call plus a `Rectangle` call becomes a bounded union.

Call-site propagation MUST NOT spray facts into every workspace method that shares a selector. It requires a resolved or bounded dispatch target set.

## 12.9 Recursive and mutually recursive calls

Callable summaries MUST be solved to a fixed point.

Use a work-list algorithm and widening:

```text
seed exact/local facts
enqueue changed callable
recompute summary
if summary changed:
    enqueue reverse dependents
if a union exceeds MAX_SHAPE_UNION:
    widen to Unknown
```

The solver MUST terminate for recursive graphs.

## 12.10 Use-site constraints

When a parameter/value remains unknown, observed sends can form a *shape constraint*:

```phalcom
process(_ x) {
  x.render()
  x.bounds
}
```

This means “candidate runtime classes must respond to both selectors,” not “x has a declared interface type.”

The semantic database MAY use these constraints to rank completion candidates. It MUST label this confidence `Heuristic` and MUST NOT display it as an exact default inlay hint.

## 12.11 Reflective/dynamic sends

Computed selectors, reflective `perform`, dynamic expansions that prevent static selector reconstruction, runtime class mutation, and analogous reflective features widen the affected knowledge.

The analyzer should prefer a correct `Unknown` to a confident wrong answer.

---

# 13. Completion-target recovery

The existing `receiver_prefix` must be retired as the semantic receiver parser.

Editor buffers are often incomplete, especially immediately after typing a dot. The completion path therefore needs an error-tolerant target extractor.

Implement:

```rust
pub struct CompletionTarget {
    pub receiver_range: SourceRange,
    pub partial_member: String,
}
```

Resolution strategy:

1. use the recovered AST when it contains an expression ending at the member-access boundary;
2. otherwise use the real Phalcom lexer/token stream to scan backward with delimiter balancing;
3. map the recovered source range to the smallest AST expression that covers it;
4. if necessary, parse a receiver expression fragment through an additive parser entry point rather than writing a second grammar.

A raw regex or ASCII identifier scanner MUST NOT remain the main path.

Required examples:

```phalcom
p.
factory.make().
self.client.
users[0].
(pointFactory()).
```

All must produce a receiver expression that `SemanticDb::infer_expr` can query.

---

# 14. Completion algorithm

New conceptual API:

```rust
pub fn complete_member(
    db: &SemanticDb,
    module: ModuleId,
    receiver: ExprRef,
    context: CompletionContext,
) -> Vec<CompletionCandidate>
```

## 14.1 Exact single-class receiver

Return visible members of that class, including inherited members and side filtering. Preserve current visibility behavior.

## 14.2 Union receiver

For `A | B`:

1. members present on every candidate class rank first;
2. members present only on some candidate classes MAY be shown after them;
3. partial-union items MUST carry detail indicating coverage;
4. completion MUST not silently pretend a member is valid on every alternative.

Example detail:

```text
move(_,to)   Point | Vector — available on 2/2 candidates
normalize()  Vector — available on 1/2 candidates
```

## 14.3 Unknown receiver with use-site constraints

Query the live workspace/core class surface index for candidate classes satisfying known selector constraints. Rank by:

1. lexical/current module;
2. imported modules;
3. inferred candidate frequency/evidence;
4. core module;
5. remaining workspace.

## 14.4 Truly unknown receiver

Do not fall back to a generated merged core table.

The default should be a bounded live selector vocabulary from visible workspace/core surfaces, ranked and marked low-confidence. A setting MAY disable this fallback entirely.

## 14.5 Completion item metadata

Items SHOULD set:

- `kind`;
- selector label;
- snippet insertion;
- `detail` with owner and confidence;
- `sortText` reflecting semantic rank;
- `filterText` where selector syntax benefits from it;
- lazy documentation through `completionItem/resolve` if hover/Phaldoc is expensive.

---

# 15. Inlay type hints

## 15.1 Protocol

Advertise the standard server capability in `Backend::initialize`:

```rust
inlay_hint_provider: Some(OneOf::Right(
    InlayHintServerCapabilities::Options(InlayHintOptions {
        resolve_provider: Some(true),
        work_done_progress_options: Default::default(),
    })
)),
```

Implement:

```rust
async fn inlay_hint(
    &self,
    params: InlayHintParams,
) -> Result<Option<Vec<InlayHint>>>
```

The pinned `tower-lsp 0.20` trait supports this request.

## 15.2 Hint positions

For a simple binding:

```phalcom
let user = fetchUser()
```

render conceptually:

```text
let user: User = fetchUser()
```

The hint is not source text and MUST NOT be inserted into the program.

Hints SHOULD anchor at the end of the bound identifier/pattern leaf.

## 15.3 Hint kinds

Use `InlayHintKind::TYPE` for runtime-shape/type-style hints.

For high confidence:

```text
: String
: Point
: List<String>
```

For heuristic facts, if the user enables them:

```text
≈ Drawable
≈ Point | Vector
```

The renderer MAY use a different low-confidence prefix, but it must visibly distinguish conjecture from stable facts.

## 15.4 Default display policy

Default:

- `Exact`: show;
- `Flow`: show;
- `Interprocedural`: show;
- `Heuristic`: hide;
- `Unknown`: hide.

Obvious literal hints SHOULD be suppressible and SHOULD default to suppressed if visual noise becomes excessive.

## 15.5 Tooltip

Every hint SHOULD offer a tooltip similar to:

```text
Inferred runtime value: Point
Confidence: interprocedural
From: return value of PointFactory.make()
This is editor inference, not a Phalcom type annotation.
```

This is especially important while the language has no formal type syntax.

## 15.6 Refresh

When a cross-file semantic generation changes a visible document's hints, the server SHOULD request `workspace/inlayHint/refresh` if the client advertises support. Normal local edits will also cause VS Code to re-request the affected range.

---

# 16. Hover integration

Hover SHOULD use the same `InferredValue` renderer.

For a local binding, append a compact section:

```markdown
**Inferred runtime value:** `Point`  
Confidence: flow
```

For a method, hover MAY add an inferred return summary:

```markdown
**Observed return:** `Point | None`
```

Do not allow hover and inlay hints to compute their own independent facts.

---

# 17. LSP backend changes

## 17.1 `phalcom-lsp/src/lib.rs`

- add `semantic` and `inlay_hints` modules;
- update stale module documentation to reflect current capabilities;
- remove `core_table` export after migration completes.

## 17.2 `phalcom-lsp/src/backend.rs`

Add:

- `SemanticDb`;
- `ServerConfig`;
- inlay capability/handler;
- `did_change_configuration`;
- `did_change_watched_files`;
- `did_change_workspace_folders`;
- initialization-options parsing;
- semantic refresh logic.

Completion becomes conceptually:

```rust
let target = completion::target_at(doc, position)?;
let value = self.semantic.infer_expr(uri, target.receiver_range);
let items = completion::members_for(value, ...);
```

It MUST no longer instantiate `ConstructResolver`.

## 17.3 `phalcom-lsp/src/documents.rs`

Add file version/revision and expose immutable snapshots without holding a DashMap guard through expensive semantic work.

Prefer:

```rust
pub struct DocumentSnapshot {
    pub text: Arc<str>,
    pub parse: Arc<Parse>,
    pub line_index: Arc<LineIndex>,
    pub revision: FileRevision,
}
```

## 17.4 `phalcom-lsp/src/index.rs`

Either:

- migrate class/surface storage into `semantic::surface`, leaving definition/reference indexes here; or
- make `WorkspaceIndex` a lower-level component of `SemanticDb`.

Do not extend the existing class map with ad-hoc inferred values.

## 17.5 `phalcom-lsp/src/completion.rs`

Retire:

- `ConstructResolver`;
- `ReceiverResolver` as the final abstraction;
- `class_of_construct`;
- `receiver_prefix` as the primary target parser;
- `CoreTable` fallback.

Keep/reuse:

- selector snippet rendering;
- visibility filtering;
- receiver side distinctions;
- deterministic ordering.

## 17.6 `phalcom-lsp/src/core_table.rs`

Delete after core/native source migration.

## 17.7 New tests

Add at minimum:

```text
tests/stage6_semantic_local.rs
tests/stage6_semantic_calls.rs
tests/stage6_modules.rs
tests/stage6_completion_chains.rs
tests/stage6_inlay_hints.rs
tests/stage6_core_surface.rs
```

Keep the consolidated `tests/integration.rs` target or evolve it consistently with repository test policy.

---

# 18. VS Code client integration

The extension should remain a thin standard LSP client.

## 18.1 `extension.ts`

Enhance `LanguageClientOptions`:

```ts
const clientOptions: LanguageClientOptions = {
  documentSelector: [{ scheme: "file", language: "phalcom" }],
  initializationOptions: readInitializationOptions(),
  synchronize: {
    configurationSection: "phalcom",
    fileEvents: workspace.createFileSystemWatcher("**/*.ph")
  }
}
```

The exact `vscode-languageclient` API shape should be checked against the pinned dependency during implementation; the intent is normative: configuration and workspace source changes must reach the server.

Register commands:

```text
Phalcom: Restart Language Server
Phalcom: Show Language Server Output
```

A configuration change to inference/hint policy SHOULD be forwarded without restart. A change to the executable path MAY require a controlled client restart.

## 18.2 Server binary resolution

Current behavior is configured path or `"phalcom-lsp"` on `$PATH`.

For a distributable extension, resolve in this order:

```text
1. explicit `phalcom.lsp.serverPath`
2. bundled platform-specific server binary
3. `phalcom-lsp` on PATH (development fallback)
```

If launch fails, show one actionable error with the resolved path and a link/command to open the language-server output channel.

## 18.3 Proposed settings

```json
{
  "phalcom.lsp.enabled": true,
  "phalcom.lsp.serverPath": "",
  "phalcom.lsp.sysrootPath": "",
  "phalcom.analysis.mode": "workspace",
  "phalcom.inlayHints.types": "stable",
  "phalcom.inlayHints.suppressObvious": true,
  "phalcom.completion.unknownReceiver": "constrained",
  "phalcom.trace.server": "off"
}
```

Semantics:

- `analysis.mode`: `local | workspace`; default `workspace`.
- `inlayHints.types`: `off | stable | all`; `stable` excludes heuristic facts.
- `suppressObvious`: hides `let x = "..." : String`-style hints when desired.
- `completion.unknownReceiver`: `off | constrained | workspace`; default `constrained`.
- `sysrootPath`: optional core-source location.
- `trace.server`: standard client/server protocol logging control.

Do not expose dozens of inference algorithm knobs initially.

## 18.4 No custom VS Code inlay provider

The extension MUST NOT duplicate server hints through the direct VS Code API. Once the LSP advertises `textDocument/inlayHint`, the language client is the integration layer.

---

# 19. File watching and workspace lifecycle

The server MUST support:

- initial multi-root workspace folders;
- folders added/removed after initialization;
- `.ph` file create/change/delete notifications;
- open unsaved buffers overriding on-disk versions;
- closed files reverting to disk-backed state;
- deletion removing all semantic/index contributions.

For every file the priority is:

```text
open document buffer > current disk file > absent
```

A close MUST not leave stale unsaved semantic facts in the workspace database. The current implementation keeps the last indexed contents after close; the semantic version should re-read/reindex disk on close when the open buffer differed.

---

# 20. Performance requirements

Initial targets for a warm medium workspace:

| Operation | Target |
|---|---:|
| exact/local member completion | p95 < 30 ms |
| inlay hints for visible range | p95 < 50 ms |
| hover semantic augmentation | p95 < 30 ms |
| local semantic update after edit | p95 < 100 ms |
| dependent recomputation | bounded; must not block request handling indefinitely |

These are engineering targets, not language semantics. Measure them before enforcing hard CI thresholds.

Rules:

- never run whole-workspace inference synchronously in a completion request;
- cache callable summaries by semantic generation;
- use reverse dependencies for invalidation;
- cap unions and candidate sets;
- avoid holding DashMap/RwLock guards while parsing or solving;
- cancellation SHOULD be honored for long workspace requests;
- workspace scanning SHOULD move off the initialization critical path once measured startup size justifies it.

---

# 21. Correctness rules

1. A stale semantic fact MUST never survive a newer file revision.
2. An exact hint MUST have exact/flow/interprocedural provenance; heuristic structural matching is not exact.
3. Dynamic/reflective operations widen knowledge rather than silently retaining stale precision.
4. Class identity is module-qualified.
5. Selector identity remains ADR-0012 canonical.
6. Private/protected/internal visibility is applied after semantic candidate resolution.
7. Open buffers dominate disk.
8. Native return values without contracts are `Unknown`.
9. Unknown does not generate a diagnostic.
10. Completion and inlay hints query the same semantic snapshot.

---

# 22. Test specification

## 22.1 Pure semantic unit tests

Required fixtures:

- literal → class mapping;
- binding/reassignment;
- union join and widening;
- destructuring projection;
- loop element facts;
- field writes/reads;
- source method return summary;
- recursion fixed point;
- call-site parameter union;
- resolved constructor vs non-constructor class-side method;
- visibility;
- import graph;
- cross-file superclass resolution;
- stale-revision invalidation;
- native unknown-return behavior.

## 22.2 LSP JSON-RPC integration tests

Drive `tower-lsp::LspService` over the existing in-process duplex harness.

Required scenarios:

```phalcom
let s = "hello"
s.
```

Must complete String members and return a stable `: String` inlay hint.

```phalcom
let p = Point.new()
p.
```

Must complete `Point` instance members.

```phalcom
factory.make().
```

Must use a source method return summary.

```phalcom
consume(_ x) { x. }
consume(Point.new())
```

Must infer the parameter when dispatch is unambiguous.

```phalcom
class Service {
  @constructor
  new() { _client = Client.new() }
  run() { _client. }
}
```

Must infer the field receiver.

Cross-file fixture must verify module-qualified same-named classes do not contaminate each other.

An edit adding a method to a class in an unsaved buffer must make that method appear in completion without saving/regenerating anything.

## 22.3 Core/native regression tests

- `core-table.json` is absent.
- `gen-core-table` is absent or no longer part of editor intelligence.
- core `.ph` method addition/removal changes the indexed surface.
- runtime native registrations and canonical native declarations are validated against each other.
- an opaque native return without a declared semantic contract remains `Unknown`.

## 22.4 VS Code E2E tests

Replace the sample extension test with tests that launch the actual extension and LSP.

Use VS Code command APIs to request:

- completion;
- hover;
- definition;
- inlay hints;
- diagnostics where practical.

Test server path resolution and restart behavior.

The current sample test is not an acceptance gate for language integration.

**Current reference:** [`extension.test.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/test/suite/extension.test.ts#L1-L25)

## 22.5 Repository gates

At minimum:

```sh
scripts/test.sh lsp
npm --prefix tools/vsphalcom test
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
cargo fmt --check
```

Use the repository's broader verification script for final integration.

---

# 23. Implementation phases

## Phase A — semantic foundation

Write set:

```text
phalcom-lsp/src/semantic/**
phalcom-lsp/src/documents.rs
phalcom-lsp/src/index.rs
phalcom-lsp/src/lib.rs
phalcom-lsp/tests/**
```

Deliver:

- semantic IDs;
- module graph;
- source surfaces;
- local exact/flow inference;
- revisioned invalidation;
- no user-visible inlays yet.

Exit criteria: local facts are queryable and deterministic; same-name classes across modules remain isolated.

## Phase B — completion migration

Write set additionally:

```text
phalcom-lsp/src/completion.rs
phalcom-lsp/src/backend.rs
```

Deliver:

- expression completion targets;
- constructor-safe inference;
- field inference;
- source return summaries;
- chained completion;
- parameter call-site propagation;
- union ranking.

Delete `ConstructResolver` when parity tests pass.

Exit criteria: all existing Stage 3 tests pass plus chain/field/parameter tests.

## Phase C — inlay hints

Add:

```text
phalcom-lsp/src/inlay_hints.rs
```

Deliver standard `textDocument/inlayHint`, tooltip/provenance, configuration policy, refresh.

Exit criteria: stable hints agree with completion's inferred receiver classes.

## Phase D — live core/native source

Write set may include:

```text
phalcom-core/core/core.ph
phalcom-core/src/universe/primitives.rs
phalcom-core/bin/gen-core-table/**
phalcom-lsp/src/core_table.rs
tools/vsphalcom/src/generated/core-table.json
```

Deliver:

- core source imported into semantic DB;
- canonical native declaration surface;
- runtime/declaration validation;
- delete generated JSON and generator dependency from LSP.

Exit criteria: no editor intelligence consumes `core-table.json`.

## Phase E — VS Code production integration

Write set:

```text
tools/vsphalcom/src/extension.ts
tools/vsphalcom/package.json
tools/vsphalcom/README.md
tools/vsphalcom/src/test/**
packaging/release configuration
```

Deliver:

- settings synchronization;
- file watchers;
- restart/output commands;
- bundled server resolution;
- real E2E tests;
- documentation.

Exit criteria: a clean extension install can start the server without requiring a manually configured PATH, subject to the chosen release packaging model.

## Phase F — hardening

Deliver:

- multi-root lifecycle tests;
- closed-file disk reversion;
- cancellation/performance instrumentation;
- stale-fact race tests;
- documentation cleanup;
- remove obsolete Stage 1/2 status text.

---

# 24. Ratified design decisions

| Decision | Resolution |
|---|---|
| What is an inferred “type”? | Advisory runtime value shape/class knowledge, not a language type |
| Where does inference live? | `phalcom-lsp/src/semantic` initially |
| Does the LSP link the VM? | No |
| How are classes identified? | `(ModuleId, class name)` |
| How are selectors identified? | Existing canonical comma-form |
| What replaces `ConstructResolver`? | Expression inference query against `SemanticDb` |
| What replaces `core-table.json`? | Live core source + canonical native declarations |
| How are opaque native returns handled? | `Unknown` unless canonical semantic contract exists |
| How are uncertain values represented? | Bounded unions + confidence/provenance |
| Default inlay confidence | Exact/flow/interprocedural; heuristic hidden |
| Does inference emit type errors? | No |
| Unknown completion fallback | Live constrained/workspace surface, not generated merged core |
| Does TypeScript implement inference? | No; VS Code remains a standard LSP client |
| Must parser become incremental now? | No; semantic invalidation must be incremental, parser may remain full-file initially |
| Future explicit typing | Becomes stronger semantic input without replacing architecture |

---

# 25. Acceptance criteria

The unit is complete only when all of the following are true:

- typing `let s = "x"` can produce a `String` inlay hint and `s.` can complete the live String surface;
- completion works after recoverable chained receivers such as `factory.make().`;
- source method return summaries affect callers;
- parameters can receive bounded call-site knowledge;
- fields can receive class-local assignment knowledge;
- cross-file semantic identities are module-qualified and imports are represented;
- a core source edit can affect completion without regenerating JSON;
- `core-table.json` is no longer an LSP dependency;
- opaque native-return uncertainty is represented honestly;
- inlay hints use standard LSP;
- VS Code settings are synchronized to the server;
- the extension has real client/server E2E tests;
- existing diagnostics, definition/references, hover, completion visibility behavior, and semantic tokens remain green;
- `scripts/test.sh lsp` is green;
- no `phalcom-core` VM dependency is added to `phalcom-lsp`.

---

# 26. Forward compatibility with a future Phalcom type system

When explicit type syntax lands, the semantic engine should add a fact origin such as:

```rust
FactOrigin::ExplicitAnnotation(SourceRange)
```

and rank it above inference. The rest of the architecture remains unchanged:

```text
explicit annotation
      ↓
ValueKnowledge
      ↓
completion / inlay / hover / diagnostics
```

At that point the compiler may choose to enforce some facts. Until then, the LSP stays descriptive rather than prescriptive.

This is the core reason to build one semantic database now rather than a collection of editor-only type guesses.
