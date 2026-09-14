# Phalcom LSP — Next-Agent Implementation Handoff

## Baseline

Work against repository `aureat/phalcom-lang`, `main` commit:

```text
8f41ee4a7029f0617930cb01348454a111d072fb
checkpoint: commit live semantic workspace changes
```

Before editing, run:

```sh
git rev-parse HEAD
git status --short
```

If `HEAD` has moved, inspect the diff from the baseline only for the files named by the current spec. Do not rescan the whole repository.

## Authority/order

Implement the three supplied specs in this order:

1. `01-fixed-point-callable-parameter-solver.md`
2. `02-module-graph-incremental-invalidation.md`
3. `03-receiver-hover-definition-phaldoc.md`

The existing repository design package remains authoritative where these specs do not explicitly refine a now-obsolete handoff statement:

```text
patchwork/phalcom-lsp-live-semantic-design-package/
  PHALCOM_LIVE_SEMANTIC_INTELLIGENCE_IMPLEMENTATION_SPEC.md

patchwork/phalcom-lsp-testing-kit/
  POST_LANDING_TESTS_TO_ADD.md
  TEST_MATRIX.md
  MERGE_GATES.md
```

The new specs are based on the current committed implementation, which is ahead of the older handoff: a bounded fixed-point loop, `Arc<Program>` snapshots, callable dependencies, import-resolution refresh, `return_for_callable`, and a partial receiver-aware hover implementation already exist. Do not reimplement those features from scratch.

## First action: apply the test patch

From repository root:

```sh
git apply --check /path/to/phalcom-lsp-regression-tests.patch
git apply /path/to/phalcom-lsp-regression-tests.patch
```

The patch is tests/fixtures only. It is intentionally designed to expose remaining semantic bugs. Do not modify expected outcomes to make current behavior green.

## Mandatory targeted-read discipline

Do **not** preload all LSP files into context.

For each spec:

1. Read only the exact files/functions listed under “Read only these files first”.
2. Use targeted searches for names, never broad repository dumps. Examples:

```sh
rg -n "rebuild_state|parameter_facts_for_program|summaries_for_surface" phalcom-lsp/src/semantic
rg -n "semantic_definition_locations|hover_at|goto_definition" phalcom-lsp/src/backend.rs
rg -n "harvest_doc_for_selector|SelectorSite" phalcom-lsp/src/hover.rs
```

3. Open 80–200 lines around the matched function, not the entire file unless the file is already small.
4. Write the smallest coherent patch for the current step.
5. Run the focused test for that step immediately.
6. Only then read the next function/file.

Do not “understand the whole codebase first.” The specs already identify the implementation boundary.

## Known current defects to keep in mind

These are confirmed against the baseline commit:

### Solver/parameters

- `semantic/mod.rs::rebuild_state` drains `InvalidationQueue` into `_affected` and then ignores it.
- It reads every `state.files` entry and recomputes the whole workspace.
- Cross-module parameter aggregation uses `BTreeMap::extend`, which overwrites identical `(CallableId, parameter)` keys instead of joining them.
- The call-site parameter walker does not seed member environments with already inferred parameter facts, preventing transitive forwarding such as `forward(value) -> sink(value)`.
- A hard `MAX_SOLVER_ROUNDS = 64` can publish a non-fixed state silently.

### Modules/invalidation

- `resolve_named_class` still has a workspace-global unique-class fallback. Remove it; module scope requires local/import/core resolution.
- `ModuleGraph::refresh_resolutions()` returns changed importers but callers ignore that list.
- removal can erase reverse import edges before old dependents are captured.
- import path code always uses `.with_extension("ph")`; language semantics append `.ph` only when no extension is present.
- `Backend::scan_workspace` repeatedly invokes `update_file`, and each current update globally rebuilds semantics.

### Hover/definition/docs

- inherited hover uses the receiver class name rather than `member.callable.owner`.
- inherited definition resolution calls `member_surface` instead of inheritance-aware `receiver_member`.
- receiver-qualified definition can fall back to all globally indexed selector definitions when semantic resolution fails.
- `return_for_selector` remains a selector-only semantic fallback.
- `SelectorSite` drops `ClassId`, so same-named module classes can become `User, User`.
- Phaldoc harvesting remains selector-keyed within a file and can attach A’s `ping()` docs to B’s `ping()`.
- class `///` docs/class hover are not implemented.

## Do not do these things

- Do not replace `SemanticDb` with `WorkspaceIndex`.
- Do not remove the global selector index entirely. It remains useful for references/workspace symbols and explicitly global navigation surfaces.
- Do not use selector text alone as return/method semantic identity.
- Do not make `phalcom-lsp` depend on the VM/runtime.
- Do not reintroduce global class visibility to preserve an old test; fix the test to use proper imports.
- Do not invent cross-module superclass syntax if the parser does not support it.
- Do not touch unrelated example/core files or perform repository cleanup.
- Do not create commits or switch branches unless explicitly asked by the user.
- Do not weaken regression tests because the current implementation fails them.

## Test sequence

After applying the test patch, establish baseline failures with focused commands. Then implement Spec 1 and make its tests green before starting Spec 2.

Recommended sequence:

```sh
cargo test -p phalcom-lsp --test integration \
  parameter_facts_from_multiple_consumer_modules_join_instead_of_overwriting
cargo test -p phalcom-lsp --test integration \
  inferred_parameter_facts_propagate_through_forwarding_calls
```

Then Spec 1 semantic units:

```sh
cargo test -p phalcom-lsp semantic::
cargo test -p phalcom-lsp --test integration workspace_semantics
```

Then Spec 2 module/invalidation tests, then Spec 3 hover/definition tests.

Final focused gate:

```sh
cargo test -p phalcom-lsp
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
```

Only after Rust LSP behavior is correct:

```sh
cd tools/vsphalcom
npm test
```

Then run broader repository gates if requested/appropriate.

## Required implementation reporting

When you finish each spec, report:

1. exact files changed;
2. exact semantic invariants implemented;
3. tests added/changed;
4. focused commands run and results;
5. any deferred issue, with an exact code anchor and reason.

Do not report “done” merely because `cargo test -p phalcom-lsp` is green if the spec’s targeted regression tests or exact invalidation checks were not run.
