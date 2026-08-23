I audited the latest repository state at `5c64793299ab7f29dd7ec4d6a4e76c33dfa14167` (`fix module identity and runtime materialization`) against Parts I–III, with particular attention to the findings from the previous audit.

The repair commit is substantive. Several earlier defects are genuinely fixed. But the implementation is still not spec-complete. I see three architectural blockers, three high-severity correctness gaps, and several medium-level semantic/incremental issues. Most importantly, the module system currently has a strong static-resolution/linking skeleton, but `CompiledProgram` is not yet actually a compiled program image, semantic declaration materialization is largely scaffolding, and the LSP is still running its legacy URI-based resolver in its production semantic-engine path.

| Previous audit finding | Latest status | Assessment |
|---|---|---|
| Real project could collide with core project `0` | Fixed | Real projects now start above reserved ID 0 |
| Root `package.ph` confinement hole | Fixed in code | Regression test does not actually exercise an escape |
| Same physical source can obtain multiple ModuleIds | Fixed | Reverse `SourceId → ModuleId` tracking added |
| `export Foo` before declaration failed | Fixed | Interface collection is now multi-pass |
| Import/body declaration collisions | Fixed | Unified namespace catches this case |
| `core` missing as resolver root | Partially fixed | Resolver knows `core`, but compilation/linking does not |
| Mixed `path` + `package` dependency spec accepted | Fixed | Strict manifest decoding added |
| Project-cycle diagnostic closed on wrong node | Fixed | Cycle slice now closes on repeated manifest |
| Per-import allocation of import-root map | Fixed | Roots are precomputed |
| Interface errors stringified immediately | Mostly fixed | `ModuleLoadError` preserves `InterfaceError`; some paths still degrade |
| Standalone file gained sibling imports | Not fixed | Existing execution path still constructs a directory-backed synthetic project |
| Duplicate declarations overwritten in interface | Not fixed | First-pass `BTreeMap::insert` still overwrites |
| Explicit namespace requirement | Not fixed / design drift | Namespace remains optional |
| Metadata target syntax | Not fixed / design drift | Implementation still uses `@!` |

## BLOCKER — `CompiledProgram` is still a linked-source plan, not a compiled program image

This is now the largest discrepancy between the implementation and the architecture.

`ProgramCompiler::compile_entry_selection()` discovers sources, builds interfaces and links them, but each `CompiledModule` still carries its source text. `compile_module()` constructs its artifact with `ModuleArtifact::empty(module)`.

And `ModuleArtifact::empty()` is literally:

```rust
Self {
    id: module.interface.module.clone(),
    declarations: Vec::new(),
    initializer: None,
    ...
}
```

So the declaration-blueprint and compiled-initializer portions of the abstraction are currently empty scaffolding.

Then `VM::run_compiled()` does this:

```rust
self.materialize_program(program)?;

for (id, compiled_mod) in &program.modules {
    if let Some(source_text) = &compiled_mod.source_text {
        self.compile_program_module_closure(id, source_text, program)?;
    }
}

self.initialize_program(program)
```

and `compile_program_module_closure()` eventually enters `compile_closure_as_with_bindings()`, which calls `parse_source(source, 0)` again. In other words, sources have already been parsed for interfaces, but are reparsed and bytecode-compiled inside the VM after program materialization.

That is materially different from the intended architecture:

```text
resolve
→ parse
→ interface discovery
→ link
→ semantic analysis
→ compile declarations + initializer
→ CompiledProgram
→ VM materialization
→ initialization
```

Part II explicitly requires a `ModuleArtifact` containing declaration metadata and initializer bytecode/compiled closure representation, and Part III consumes that artifact.

This has a second consequence: the declaration-materialization architecture has not actually landed. `RuntimeDeclarationBlueprint::Class` and `ClassBlueprint` exist, but repository search finds them consumed by materialization rather than produced by compilation. `ModuleArtifact::declarations` is always empty along this program-compilation path.

That also explains why qualified semantic declaration references are not operational. The AST supports static references such as `base.Shape`, and the linker exposes `resolve_static_symbol()`, but the ordinary class compiler still resolves a superclass through `resolve_superclass_ref()`. That helper only resolves bare references; qualified references deliberately return no local `ClassKey`. The class compiler subsequently treats an unresolved superclass as unknown/forward.

So something as fundamental as:

```phalcom
import .base

class Child is base.Shape {
}
```

has the parser/linker vocabulary needed to represent the relationship but not the completed compiler/materialization path needed to realize it.

This means semantic SCC support is currently infrastructure rather than an implemented semantic cycle model. The graph code itself is well designed—it has separate reference, semantic and runtime graphs and deterministic SCC computation—but actual declaration identities are not being materialized through those SCCs.

I would not close Parts II–III until `ProgramCompiler` produces populated artifacts, qualified declaration references are consumed by compiler/materializer code, and `VM::run_compiled()` no longer reparses source.

## BLOCKER — the new `core` root resolves, but cannot participate in a linked program

The repair correctly introduced `ImportRootTarget::Core` and reserved project ID zero. That fixes the original identity collision for normal project modules.

But the implementation currently stops at resolver-level recognition.

For an absolute `core` import, `ModuleResolver::resolve_import()` manufactures a `SourceUnit` for `ModuleId::core()` and returns it directly. `ModuleId::core()` belongs to reserved project ID zero, which intentionally has no `ResolvedProject` in `ProjectUniverse`.

`ProgramCompiler::discover_and_link()`, however, treats every resolved import uniformly:

```text
resolve import
→ add target ModuleId to pending
→ later resolver.load_interface(target)
```

`load_interface(ModuleId::core())` asks `ProjectUniverse::get_project(RESERVED)`, which cannot return a filesystem project. The linker also requires reachable import targets to have interfaces in its supplied interface map.

So the newly added test:

```rust
resolver.resolve_import(... "core" ...)
assert_eq!(resolved.id, ModuleId::core());
```

proves only resolver recognition. It does not prove:

```phalcom
import core
```

can be compiled and linked.

There is another bug in the same resolver branch: it returns the core `SourceUnit` immediately and does not reject additional path segments. Thus logically distinct input such as:

```phalcom
import core.foo
```

can resolve to the same `ModuleId::core()` instead of producing a path error.

Core needs to become a genuine non-filesystem provider/prelinked interface in the module universe. It should have a static interface that the linker can consume without asking `FilesystemSourceProvider` for project zero. If v1 has no `core.*` child modules, any nonempty path after the reserved root should be rejected.

The LSP has the same missing abstraction: `SharedModuleResolver` attempts to turn the returned `SourceUnit.display_path` into a file URI. A synthetic `<core>` source path is not a normal filesystem document.

## BLOCKER — LSP production analysis is still using the old URI/filesystem resolver

The latest LSP code contains a good `SharedModuleResolver` adapter around `phalcom_modules::ModuleResolver`. It also has a `DocumentModuleMap` that can map editor URIs to semantic module identities.

But that is not the path used by `SemanticEngine`.

`SemanticEngine::update_files_batch_inner()` still gives files URI-derived LSP identities through `ensure_lsp_for_uri()`, constructs an `available` set, and calls:

```rust
self.state.graph.update(
    file.module.clone(),
    &file.source.program,
    &available,
);
```

That is the legacy `ModuleGraph::update`, not `update_with_shared_resolver`.

And the legacy resolver explicitly says:

```rust
if dot_count == 0 {
    // Absolute logical roots need ProjectUniverse context.
    return Vec::new();
}
```

So absolute imports such as:

```phalcom
import geometry.point
import math.vector
```

cannot be resolved by the actual semantic-engine path. Nor does this compatibility resolver have authoritative dependency-alias or `expose` visibility information.

This is exactly the duplicated-meaning problem the new module crate was intended to eliminate.

`update_with_shared_resolver()` exists, but repository search found its definition rather than an integration call site. The actual worker engine shown above still calls `update()`.

This is a Part-II completion blocker. The semantic worker needs a project/universe/source-provider generation at ingestion time, must establish `URI ↔ SemanticModuleId` before indexing, and must build its graph through the shared resolver. Once that is working, the legacy URI candidate resolver should be removed rather than left as another semantic authority.

## HIGH — standalone-file and standalone-package ownership semantics remain wrong

Part I is very explicit here: a standalone module gets one execution-local identity, core/builtin visibility, and **no sibling imports**. A standalone package is instead discovered from a contiguous hierarchy of directories containing `package.ph`.

There is now an `EntryOwnership` enum in the source subsystem, which is the right abstraction. But the execution path is not using it.

For a module file outside a project, `EntrySelection::Module` still does:

```rust
let parent = file_path.parent();
let file_stem = ...;

universe.load_synthetic_root(
    file_stem,
    parent,
    file_stem,
)?;
```

That turns the file's containing directory into an ordinary source root.

Thus:

```text
scratch/
├── app.ph
└── helper.ph
```

can acquire project-like sibling resolution even though the specification expressly says standalone `app.ph` does not get `helper.ph` merely because it sits next to it.

Likewise a directly executed source inside:

```text
foo/
├── package.ph
└── tools/
    ├── package.ph
    └── run.ph
```

is not classified by the contiguous `package.ph` ownership rule when selected as a file. It falls into the synthetic-file path.

`EntryOwnership` should become the single pre-compilation classifier rather than remaining an unused description of the desired states.

## HIGH — `ModuleId` is only universe-local, but `VM.module_registry` is not

The normal project/core collision is fixed: project ID zero is reserved and normal `ProjectUniverse` nodes begin above zero. Good.

But `ResolvedProjectId` remains a graph-node number scoped to one `ProjectUniverse`. Two independently compiled programs will commonly both contain:

```text
ResolvedProjectId(1)
ModulePath(["main"])
```

Those `ModuleId`s compare equal even though they belong to unrelated project universes. This is acceptable inside the compile-time abstraction only if `ModuleId` values never cross universe boundaries without their universe.

The runtime registry does exactly that: it is VM-global and keyed solely by `ModuleId`.

The latest `materialize_program()` adds “idempotence”:

```rust
if !self.module_registry.contains_key(id) {
    allocate...
}
```

If the ID is already present, it silently reuses that runtime module object. Subsequent phases replace linked reads, exports and closure information on the reused object while retaining its existing globals and lifecycle state.

That is safe only for repeated materialization of the exact same installed program generation. The new test checks precisely that narrow case. It does not check two separately compiled projects that happen to receive the same universe-local IDs.

`VM::run_compiled()` currently accepts any `CompiledProgram`, so the restriction is not enforced.

There is a related escape hatch. `ModuleRegistry` gained the safer `register_new()`, but still exposes overwrite-style `insert()`, and `VM::create_module_with_id()` still uses that overwrite operation. `VM::create_module()` constructs its ID with `ModuleId::synthetic(logical_name)`.

Because synthetic IDs also use reserved project zero, this remains possible at the identity level:

```rust
ModuleId::synthetic("core") == ModuleId::core()
```

and the compatibility creation API can overwrite a registry record.

V1 does not need multi-program loading, but it needs an explicit invariant. Either a VM owns exactly one installed user `ProjectUniverse`/program generation and rejects a different one, or runtime keys must include a universe/program identity. I would choose the former for v1. Then delete/generalize the overwrite `insert()` path and make synthetic identities a distinct tagged domain rather than encoding them through reserved project zero.

Also, `ResolvedProjectId::from_raw()` is public, which weakens the otherwise good “opaque ID issued by ProjectUniverse” abstraction.

## HIGH — `InterfaceBuilder` is now order-independent, but duplicate declarations still overwrite

The three-pass interface refactor is the right fix for the previous export-order bug.

However, the first declaration pass still builds its namespace and `declarations` maps with ordinary insertion. A second declaration of the same name can replace the first static surface instead of producing an interface error.

The compiler may subsequently reject some duplicate declarations, but that is architecturally too late. `UnlinkedModuleInterface` is consumed by the linker and LSP as static truth; it should never represent source that its own module namespace considers ambiguous/redeclared.

The interface builder needs a single insertion operation that preserves the first declaration span and rejects every second module-namespace producer: class/class, let/let, class/let, duplicated destructuring names, imports against declarations, and so forth.

## MEDIUM — manifest identity is still conflating project name and programming namespace

The implementation now deliberately accepts an omitted `namespace` and derives it from the project name. The implementation spec uses an explicit namespace because project/distribution naming and source namespace identity were meant to remain independent.

This may be a perfectly reasonable ergonomic change, but it is a language-design change rather than merely an implementation detail.

There is also still a concrete diagnostic-data bug: `ValidatedProjectManifest.name` is assigned the normalized namespace, while `raw_name` retains the actual project name; `ResolvedProject.name` is documented as the original name for diagnostics but receives the former.

Keep these concepts unambiguously separate:

```text
distribution/project name
programming namespace
resolved graph identity
```

## MEDIUM — module/package metadata still diverges from the specified surface

The implementation continues to use:

```phalcom
@! documentation(...)
```

with target inferred from whether the source happens to be a module or package. It then has a hard-coded package-only name table:

```rust
matches!(name, "expose_policy" | "package_root")
```



The implementation spec instead introduced explicit header targets:

```phalcom
@module.documentation(...)
@package.documentation(...)
```

precisely so metadata target was explicit and orthogonal to ordinary class/member `@Attribute` syntax.

The important semantic requirement—metadata is inert—is satisfied. But `@!` and those two magical names are unratified language surface. This should be settled before source compatibility matters.

I would particularly remove `is_package_only_attribute()` unless there is an actual standardized metadata registry. Arbitrary metadata should not acquire special semantics because its string happens to equal `"package_root"`.

## MEDIUM — structured loading diagnostics are only partially preserved

Adding:

```rust
ModuleLoadError::Resolution
ModuleLoadError::Parse
ModuleLoadError::Interface
```

was the correct repair. Interface errors now survive as typed `InterfaceError`s.

Two holes remain.

`Parse` stores only a `String`, rather than the actual parser diagnostic/span, so structured parse information is still lost at the module boundary.

And exposure traversal through `load_package_surface()` still has to translate module-load failures back into a `ModuleResolutionError` surface, which causes some structured parse/interface information to degrade again.

A resolver error, source error, parse error and interface error should remain separate all the way to the compiler/LSP diagnostic adapter.

## MEDIUM — source-provider cache lifetime still has no generation model

`FilesystemSourceProvider` now correctly caches resolution, source contents, and reverse source identities, with `clear_cache()` clearing all of them.

But negative resolutions and source contents remain valid indefinitely until that global clear.

For one-shot compilation that is fine. For LSP/project sessions:

```text
resolve missing foo
→ cache ModuleNotFound
→ user creates foo.ph
```

must have a defined invalidation path.

The existing abstraction does not encode a source generation or keyed invalidation contract. Given how much work has already gone into Phalcom's incremental LSP invalidation, I would make this explicit now rather than relying on callers remembering to discard the entire provider.

The portability checks from Part I—especially case-fold-equivalent module names—also still appear absent.

## The new regression tests need strengthening

The repair tests are useful and clearly targeted at the earlier review. But several are weaker than their names suggest.

`test_confinement_violation_for_root_package` creates a completely valid ordinary root package; it does not symlink `package.ph` outside the source root, so it would not catch a regression that removed the confinement check again. `test_distinct_synthetic_modules_have_distinct_identities` compares `"mod1"` and `"mod2"` rather than two independent synthetic modules with the same display name, and does not test `"core"`. `test_import_core_resolves_reserved_root` stops at resolver output rather than compiling a project using that import.

The current suite also needs end-to-end negative or positive coverage for standalone sibling rejection, standalone-package file ownership, duplicate body declarations at the interface layer, two independently compiled programs entering one VM, qualified cross-module superclass resolution, a legal semantic SCC, and LSP absolute/dependency-alias resolution through the shared resolver.

The runtime repair tests for whole-module re-export, callable exports, export/method collisions and same-program materialization identity are good additions.

I also found no GitHub Actions workflow run or combined status attached to the latest commit through the connector, so this assessment is based on source inspection and the test code present in the repository, not an independently verified green CI run.

## Overall assessment

The implementation has moved materially forward since the previous audit. I would now consider the basic project resolver, hierarchical exposure model, import/export syntax, live linked binding representation, runtime DAG machinery, export namespace and package/module object direction to be on solid architectural ground.

I would not yet mark the three-part module implementation complete.

The priority order I would use is: first, finish the actual compile/artifact/declaration-materialization pipeline; second, make reserved `core` a real linkable external module; third, wire the LSP to the shared resolver and remove its legacy semantic resolver; then fix standalone ownership; then close the runtime universe/registry identity boundary; then make `InterfaceBuilder` fully authoritative for namespace uniqueness. The remaining metadata/manifest/cache issues can follow without forcing architectural rewrites.

The encouraging part is that none of those requires replacing the chosen module model. The abstractions are largely the right ones. The current gap is that several of the decisive abstractions—`ModuleArtifact`, `ClassBlueprint`, semantic/static references, `EntryOwnership`, and `SharedModuleResolver`—exist but are not yet the path through which the system actually operates.