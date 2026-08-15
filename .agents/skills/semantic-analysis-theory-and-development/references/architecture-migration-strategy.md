# Architecture Migration Strategy

## 1. Migrate semantic ownership, not just files

Phalcom already has substantial semantic infrastructure inside `phalcom-lsp`. The wrong migration is “move this directory into a new crate and update imports.” The real goal is to extract protocol-neutral semantic truth while preserving current behavior and allowing LSP, checker, compiler diagnostics, and future prover to consume it without circular dependencies.

**CURRENT:** semantic code depends on AST/common types and, in some files, LSP URL types and LSP-adjacent selector/performance modules. A migration should identify which dependencies are semantic necessities and which are host/application conveniences.

## 2. Anti-corruption boundary

Define a stable semantic API before moving implementation:

```text
inputs:
  canonical source/module identity
  parsed/recovered source + source text/revision
  project/module provider view
  native/core semantic contracts
  analysis configuration/budget

outputs:
  immutable SemanticSnapshot
  semantic identities/surfaces/facts/summaries
  protocol-neutral diagnostics/evidence
  semantic queries
```

LSP converts document URIs/positions to semantic source IDs and query positions, then converts results back to protocol objects. The semantic engine should not construct `Hover`, `CompletionItem`, `Location`, or workspace edits.

## 3. Strangler migration sequence

A low-risk sequence:

1. Freeze behavior with semantic/incremental regression tests around current `phalcom-lsp/src/semantic/`.
2. Extract shared value objects first: IDs, source origins, selector IDs/bridges, surfaces, fact containers.
3. Move pure algorithms: scope construction, occurrence indexing, dispatch approximation, joins, summaries.
4. Introduce a protocol-neutral source/module abstraction so LSP `Url` is an adapter concern.
5. Move worker/snapshot/invalidation architecture after dependencies are acyclic.
6. Redirect LSP handlers to the extracted API without behavior change.
7. Only then add checker-specific type domains, CFG/HIR changes, or new precision.

Do not combine architectural extraction with major semantic redesign unless tests can isolate both dimensions.

## 4. Compatibility seam

During migration, an adapter can retain old API shapes:

```rust
// temporary LSP adapter
fn semantic_db_for_lsp(...) -> Arc<SharedSemanticSnapshot>;
```

Use adapters at module boundaries, not duplicated engines. Remove old implementation only after all consumers point at the shared truth.

## 5. Current versus future representations

Preserve logical APIs while allowing representation changes. For example, current structural `ClassId` may later be interned; consumers should not parse its debug string. Current `ValueShape` remains an advisory domain even if moved into shared infrastructure. Do not rename it `Type` during extraction.

## 6. Dependency direction

Desired high-level dependency direction:

```text
phalcom-ast/common
      ^
      |
shared semantic core <--- native/core semantic contracts
      ^        ^
      |        |
    LSP      checker/prover/compiler adapters
```

The semantic core must not depend on LSP protocol or VM mutable runtime state. Compiler/runtime conformance tests can bridge both without creating a production dependency cycle.

## 7. Migration invariants

For each extraction step verify:

- same semantic identities for same source;
- same current LSP query results where semantics are intentionally unchanged;
- same incremental/full equivalence;
- no new VM requirement for editor queries;
- cancellation still cannot publish partial generations;
- no widening of dependency frontier due merely to abstraction boundaries;
- no duplicated selector/name/module resolution logic left behind.

## 8. Avoid premature “universal database” design

A future Salsa-like query system may be attractive, but first express semantic dependencies clearly in the current engine. Replacing explicit state with a framework before queries/invalidation semantics are understood merely hides dependency mistakes behind macros.

Migrate in layers: semantic ownership first, representation optimization second, query-framework adoption only if it materially simplifies dependency/revision logic.

## 9. Review questions

1. Is this migration changing ownership, semantics, or both?
2. What regression test proves no semantic behavior changed?
3. Does the extracted API contain LSP or VM-specific types?
4. Is a current advisory domain being accidentally promoted to formal typing?
5. Are selector/module identity rules still defined once?
6. Can old evidence still be retracted across the new boundary?
7. Does the shared core remain usable by batch checker without an editor/backend?
