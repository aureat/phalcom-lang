# Canonical Universe Source and Verified Bootstrap Implementation Plan

> **For agentic workers:** Execute this plan inline, task-by-task, with a commit after each cohesive task. Do not delegate work.

**Goal:** Make canonical universe source the complete, typed, documented presentation of the core surface and make bootstrap verify and compile that same source against native descriptors.

**Architecture:** Extend the existing AST/source-provider path with explicit declaration bodies, canonical selector projection, and a VM-free universe census/source index. Bootstrap loads provider units once, validates source/native identity and signature parity before installation, then installs descriptor-backed primitives and compiles the verified source units. Runtime and tooling consume the resulting source-backed metadata; legacy static source and class overlays stop being authoritative.

**Tech Stack:** Rust workspace, Phalcom AST/parser/compiler/VM, builtin source provider, generated native metadata, Phalcom `.ph` universe modules.

**Spec:** `docs/work/analyses/typing/phalcom-universe-source-and-verified-bootstrap-completion-implementation-spec.md`

## Global Constraints

- Preserve unrelated dirty and untracked files, especially the user-provided work specs.
- Canonical member identity is `(owner UniverseKey, dispatch side, encoded selector)`.
- `@native` declarations have no executable source body; `@native` source bodies are rejected unless explicitly supported as reference-only metadata.
- Authored `_$` methods require `@internal`; native authored `_$` methods require both `@internal` and `@native`.
- Parse each canonical universe module once per bootstrap and verify the AST that is compiled.
- Source/native mismatches fail before native installation or source execution.
- Descriptor-only installation is the default; legacy primitive installation remains only behind an explicit compatibility mode until all descriptor coverage is proven.
- No intermediate test or verification commands; perform formatting, tests, clippy, and acceptance checks after implementation is complete.

### Task 1: Source declaration infrastructure

**Files:**
- Modify: `phalcom-ast/src/ast.rs`
- Modify: `phalcom-ast/src/parser.rs`
- Modify: `phalcom-core/src/compiler/attributes.rs`
- Modify: `phalcom-core/src/compiler/lib/class_decl.rs`
- Modify: affected AST/compiler tests

Implement `BuiltinAttr::Internal`, explicit `MemberBody::Declaration`, parser support for declaration-only methods/getters/setters, and class-level `@native`. Enforce privileged placement and visibility rules during compilation. Preserve `None`’s immediate public binding when a native class presentation is compiled. Route all source member identities through the shared AST selector projection.

Commit: `feat(ast): support verified native source declarations`

### Task 2: Canonical source census and verification model

**Files:**
- Create/modify: `phalcom-semantic/src/core_surface/**` or the existing source-index owner
- Modify: `phalcom-core/src/native/source.rs`
- Modify: `phalcom-core/src/native/verify.rs`
- Modify: `phalcom-modules/src/builtin.rs`
- Add: VM-free census and source/native parity tests

Add deterministic module/class/member census rows, canonical `UniverseSourceIndex`, reverse descriptor coverage, class/superclass checks, side/visibility/signature checks, and structured mismatch categories. Reuse provider-parsed units so the verifier does not independently re-read source files.

Commit: `feat(native): verify canonical universe source and descriptors`

### Task 3: Canonical universe source migration

**Files:**
- Modify: `phalcom-core/core/universe/src/object/*.ph`
- Modify: `phalcom-core/core/universe/src/scalar/*.ph`
- Modify: `phalcom-core/core/universe/src/callable/*.ph`
- Modify: `phalcom-core/core/universe/src/option/*.ph`
- Modify: `phalcom-core/core/universe/src/collections/*.ph`
- Modify: `phalcom-core/core/universe/src/errors/*.ph`
- Modify: `phalcom-core/core/universe/src/concurrency/*.ph`
- Modify: `phalcom-core/core/universe/src/reflection/**/*.ph`
- Modify: `phalcom-core/core/universe/src/package.ph` and package files
- Modify: `phalcom-modules/src/builtin_interface.rs`

Replace placeholder shells with one canonical class presentation per runtime class. Add native/internal anchors for the generated descriptor surface, retain derivable source bodies, add generic/type signatures, and document modules/classes/members. Remove each corresponding hard-coded interface injection as its source declaration becomes authoritative. Resolve physical/catalog ownership conflicts and preserve special globals.

Commit source migrations in reviewable waves: object/scalar, callable/option, collections, errors/concurrency, reflection/typing reflection, and package exports.

### Task 4: Provider-based parse-once bootstrap

**Files:**
- Modify: `phalcom-core/src/vm/bootstrap.rs`
- Modify: `phalcom-core/src/vm/mod.rs`
- Modify: `phalcom-core/src/modules/compile.rs`
- Modify: `phalcom-core/src/modules/builtin_materialize.rs`
- Modify: `phalcom-core/src/native/install.rs`
- Modify: `phalcom-core/src/universe/primitives.rs`
- Modify: `phalcom-core/src/universe/**`

Replace the static `include_str!` source list with `BuiltinProjectSourceProvider::new(Universe)`. Load and parse canonical units once, build the source index, preflight source/native parity, install descriptors exactly once, and compile the verified units. Make descriptor-only startup the default while retaining explicit legacy mode only where required by migration. Remove fallback authority once coverage is closed.

Commit: `feat(vm): bootstrap canonical universe from verified source`

### Task 5: Runtime typing, reflection, and LSP convergence

**Files:**
- Modify: `phalcom-core/src/typing/registry.rs`
- Modify: `phalcom-core/src/typing/side_table.rs`
- Modify: `phalcom-core/src/modules/materialize.rs`
- Modify: `phalcom-lsp/src/semantic/core_source.rs`
- Modify: `phalcom-lsp/src/semantic/surface.rs`
- Modify: LSP definition/hover/completion consumers

Register installed native methods against source-backed semantic metadata, remove synthetic core/native overlays, and expose actual universe source for navigation and documentation. Keep VM-free source access for LSP.

Commit: `feat(tooling): consume source-backed universe metadata`

### Task 6: Final acceptance

Run repository-standard formatting, workspace tests, clippy, source-census/bootstrap parity tests, and the acceptance gates from the implementation specification. Review status/diff to separate passing implementation scope from baseline/unrelated or deferred scope. Commit only any final cohesive fixes; leave user-owned docs untouched.

Commit: `chore: close universe source and bootstrap verification gates`
