# Phalcom LSP Semantic Retirement — Closure Record

Date: 2026-08-28

Status: implementation complete; acceptance is the permanent retirement verification workflow passing at the merge candidate revision.

## Architectural result

`phalcom-semantic` is the sole implementation of Phalcom static semantics.

`phalcom-lsp` is now a protocol, live-source, scheduling, workspace-discovery, publication, and presentation layer. It does not own a parallel semantic database, type/value inference engine, scope graph, module graph, dispatch resolver, or import resolver.

The canonical data flow is:

```text
phalcom-ast
    -> recovered source products
phalcom-modules
    -> project/source/module identity + linking
phalcom-semantic::SemanticWorkspaceSession
    -> canonical semantic products
    -> Arc<phalcom_semantic::SemanticSnapshot>
phalcom-lsp::SemanticPublication
    -> RequestContext
    -> compiler-owned editor queries
    -> LSP protocol rendering
```

## Closed duplicate-semantic surfaces

The retirement removed or prohibited the former parallel semantic architecture, including:

- `phalcom-lsp/src/semantic/` as a semantic implementation;
- `phalcom-lsp/src/index.rs` as parallel semantic truth;
- LSP-owned semantic database/engine types;
- LSP-owned scope/module graph and dispatch implementations;
- compiler-to-LSP canonical identity translation bridges;
- request-time import resolution and semantic filesystem reconstruction;
- syntax-fabricated definition/reference identity;
- the final `references` fallback that treated an import token as a semantic answer when canonical compiler references were unavailable.

`phalcom-lsp/tests/semantic_boundary.rs` mechanically rejects representative forms of these regressions.

## Canonical worker and publication model

`phalcom-lsp/src/analysis_service.rs` owns scheduling around one persistent `phalcom_semantic::SemanticWorkspaceSession`.

The worker may discover/refresh source files, coalesce revisions, construct `WorkspaceSourceBatchMutation` values, and call the semantic session. The compiler/module layers own project/module identity, import linking, invalidation, typing, dispatch, inference, and semantic generation.

`phalcom-lsp/src/publication.rs` contains only the current immutable `Arc<SemanticSnapshot>` publication. It is not a semantic database.

Each `RequestContext` pins one live `DocumentSnapshot` and one immutable compiler snapshot and classifies source coherence as `Exact`, `Stale`, or `Unmapped`. Semantic requests fail closed when canonical source products cannot be trusted rather than rebuilding semantics inside the request handler.

## Imported binding correctness

The retirement exposed and repaired an important cross-module identity bug.

For:

```phalcom
// shapes.ph
class Circle {}
export Circle

// main.ph
from .shapes import Circle
let circle = Circle
```

both the imported name in the import declaration and the later `Circle` use carry the canonical exported declaration identity from `shapes.ph`.

Consequences:

- imported declarations participate in canonical semantic analysis and type inference;
- go-to-definition crosses the module boundary to the exporter;
- references use compiler target identity rather than textual/local-import identity;
- the importing token is a reference to the external declaration, not a second definition of it.

For an unresolved selective import, the compiler may still publish a local binding identity. Its import declaration and local uses therefore remain connected without fabricating a nonexistent external target. If linking later establishes the export, the canonical target becomes the external declaration.

## Module query provenance

The immutable compiler snapshot publishes canonical module query products, including:

- linked and unlinked interfaces;
- importer + written-path resolution;
- module/source provenance;
- source-to-module and display-path-to-module lookup.

LSP module-path navigation consumes those products. It does not infer module identity from URI spelling.

## Test ownership

`phalcom-lsp` intentionally has `autotests = false`. The semantic-boundary suite now verifies that every top-level `tests/*.rs` file is either an explicit Cargo test target or included by the registered integration harness. This prevents future semantic/LSP regressions from existing in the tree without ever running.

## Durable regression anchors

The retirement is guarded across all three layers.

### `phalcom-semantic`

- imported binding use resolves to the exported declaration rather than the local import site;
- imported class identity participates in expression type inference with declaring-module identity;
- editor definition sites exclude a local import declaration when the target is external;
- relative import alias/path/provenance are published by immutable module query products.

### `phalcom-modules`

- a standalone importer survives while a relative sibling is unresolved and is repaired when the sibling is discovered;
- module/linker tests enforce explicit export/import identity and canonical resolution.

### `phalcom-lsp`

- `module_navigation` verifies canonical relative-module and selective-export navigation;
- `imported_binding_resolution` verifies cross-module imported declaration/use navigation and compiler-owned unresolved import bindings;
- `semantic_boundary` enforces the single-world architecture and test registration.

## Permanent acceptance gate

The merge candidate must pass `.github/workflows/lsp-retirement-final-verification.yml`, which runs:

```text
cargo fmt --all -- --check
git diff --check
compile phalcom-semantic / phalcom-modules / phalcom-lsp
core callable compatibility tests
full phalcom-semantic suite
full phalcom-modules suite
focused canonical module-navigation regression
full phalcom-lsp suite
explicit semantic-authority boundary suite
```

A green build alone is not sufficient closure. The semantic, module, LSP, and architecture-boundary tests must all pass on the same final revision. The accepted workflow run must report the exact merge-candidate SHA as its `head_sha`; a run against an earlier branch revision does not satisfy this gate.

## Post-retirement rule

When an editor feature needs information that is semantic in nature, add or extend a protocol-neutral compiler-owned product/query in `phalcom-semantic` or `phalcom-modules`, publish it in the immutable snapshot, and adapt it in `phalcom-lsp`.

Do not restore an LSP-local semantic fallback because a compiler query is temporarily incomplete. Missing compiler coverage is a compiler semantic task, not permission for a second semantic world.
