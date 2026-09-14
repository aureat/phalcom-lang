# Phalcom Language Intelligence — Repository Analysis

**Repository:** `aureat/phalcom-lang`  
**Snapshot:** `main` at `5b6d67be93d6167558931a5c5dae3ae69959c9c4` (2026-08-11)  
**Scope:** `phalcom-lsp`, `tools/vsphalcom`, and the core/native surfaces that currently feed editor intelligence  
**Purpose:** establish the current implementation state, identify the architectural gaps blocking live dynamic inference and inlay hints, and define the constraints the implementation specification must respect.

---

## 1. Executive assessment

The Phalcom language-server effort is substantially more complete than the older planning documents imply. The current `phalcom-lsp` already implements the original five stages: live diagnostics, workspace symbol indexing with definition/reference queries, receiver-aware completion, hover/Phaldoc, and semantic tokens. The VS Code extension has already been reduced to a thin `vscode-languageclient` launcher and starts `phalcom-lsp` by default.

The remaining problem is not basic LSP plumbing. It is the absence of a shared semantic model.

Today, completion can recover only a narrow notion of a receiver class. The concrete resolver recognizes a capitalized class object, `self`, or a local variable whose most recent visible assignment looks like a call on a capitalized identifier. The resulting class name is then used to look up selectors. This works for cases such as `let p = Point.new(); p.` but it is not general value inference: parameters, fields, method-return values, chained sends, collection element shapes, imported modules, cross-file inheritance, and expression receivers are not semantically modeled.

The builtin side is also still static. `phalcom-lsp` embeds a generated `core-table.json`; that file is produced by parsing `core.ph` and text-scanning Rust primitive-registration macros. Consequently, changing core/native method surfaces requires regeneration and rebuilding rather than flowing through the same live workspace model as ordinary Phalcom source.

The recommended next step is therefore a **VM-free semantic database inside `phalcom-lsp`**. It should infer *runtime classes and value shapes* rather than introducing a hidden static type system. Completion, inlay hints, hover, and later signature help should all query this database. The generated core table should be retired in favor of live core-source indexing plus one canonical, non-generated description of native primitive surfaces.

---

## 2. Current architecture

### 2.1 LSP crate boundary

`phalcom-lsp` is intentionally VM-free. Its dependency set includes `phalcom-ast`, `phalcom-common`, `tower-lsp`, Tokio, Serde, and DashMap, but not `phalcom-core`.

**Repository reference:** [`phalcom-lsp/Cargo.toml`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/Cargo.toml#L1-L40)

This boundary is sound and should remain. Editor intelligence needs the parser and semantic descriptions of runtime surfaces; it does not need a live VM, heap, bytecode executor, or user-code evaluation.

The binary entry point is correspondingly thin: it constructs a `Backend`, wires stdin/stdout through `tower-lsp`, and delegates all behavior to the library crate.

**Repository reference:** [`phalcom-lsp/src/main.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/main.rs#L1-L25)

### 2.2 Backend and protocol capabilities

`Backend` currently owns a `Client`, `DocumentStore`, and `WorkspaceIndex`. Initialization scans workspace roots synchronously and advertises full-document synchronization, UTF-16 positions, definition, references, workspace symbols, completion, hover, and full semantic tokens.

**Repository reference:** [`phalcom-lsp/src/backend.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/backend.rs#L390-L500)

Abridged current capability shape:

```rust
completion_provider: Some(CompletionOptions { ... }),
hover_provider: Some(HoverProviderCapability::Simple(true)),
semantic_tokens_provider: Some(...),
```

There is no `inlay_hint_provider`, and no `LanguageServer::inlay_hint` implementation.

This is a protocol gap rather than a library limitation. The pinned `tower-lsp 0.20` API already exposes the standard `textDocument/inlayHint` handler, so adding inlay hints does not require changing frameworks solely for this feature.

### 2.3 Open-document model

Every open document stores:

- the complete source text;
- a recovered `phalcom_ast::parser::Parse`;
- a `LineIndex`.

Every `didOpen` and `didChange` rebuilds all three from the full source text.

**Repository reference:** [`phalcom-lsp/src/documents.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/documents.rs#L1-L110)

This is a good parse-cache foundation, but it has no semantic revision, inferred-fact cache, dependency graph, or immutable semantic snapshot. As inference becomes interprocedural, recomputing everything directly in request handlers would become both expensive and difficult to invalidate correctly.

### 2.4 Workspace index

`WorkspaceIndex` correctly keeps selector identity separate from class identity. Definitions and references use ADR-0012 comma-form selectors, while classes are keyed by `(file URI, class name)` rather than a global bare name.

**Repository reference:** [`phalcom-lsp/src/index.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/index.rs#L150-L350)

That change prevents unrelated same-named classes in different modules from being merged. However, the current implementation deliberately does not resolve `Statement::Import`, so the URI is only a local module proxy. Cross-file superclass resolution and module-qualified values are therefore unavailable to completion.

**Repository reference:** [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L585-L705)

### 2.5 Completion and receiver resolution

The current completion design has a useful seam: the `ReceiverResolver` trait. The only implementation is `ConstructResolver`.

**Repository reference:** [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L45-L145)

It recognizes three receiver forms:

1. a capitalized bare identifier such as `Point.` → class object;
2. `self.` → current lexical class instance;
3. a lowercase local identifier that `resolve_var_class` can associate with a capitalized call expression.

The textual completion target extractor only scans an ASCII identifier immediately before `.`. It does not represent the receiver as an expression. Consequently, forms such as the following are outside the model:

```phalcom
makePoint().
self.client.
clients[0].
factory.build().service.
(module.member).
```

The local assignment heuristic is also broader than its name suggests. `class_of_construct` accepts any method call whose direct receiver is a capitalized variable; it does not prove that the selector is a constructor or that the call returns an instance of that class.

**Repository reference:** [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L335-L445)

Abridged current logic:

```rust
if let Expr::MethodCall(m) = expr {
    if let Expr::Var { value, .. } = &m.object { ... }
}
```

This means `let x = SomeClass.parse(...)` can be classified as `SomeClass` even when `parse` returns another runtime object. The behavior is acceptable as a first heuristic but should not be the semantic basis of inlay hints.

When a receiver class cannot be resolved, completion falls back to `CoreTable::all_members()`, i.e. the merged builtin surface.

**Repository reference:** [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L335-L445)

### 2.6 Builtin/core selector source

`CoreTable` embeds a generated JSON artifact at build time:

```rust
const CORE_TABLE_JSON: &str =
    include_str!("../../tools/vsphalcom/src/generated/core-table.json");
```

**Repository reference:** [`phalcom-lsp/src/core_table.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/core_table.rs#L1-L180)

The generator combines two sources:

- `phalcom-core/core/core.ph`, parsed with `phalcom-ast`;
- `phalcom-core/src/universe/primitives.rs`, scanned as Rust source text for primitive-registration macros.

**Repository reference:** [`phalcom-core/bin/gen-core-table/main.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/bin/gen-core-table/main.rs#L1-L260)

This solved the original static-extension problem, but it is now the principal obstacle to a genuinely live semantic model. The user-authored core classes are already real Phalcom source. For example, `core.ph` defines `Object`, `Class`, `Error`, `Number`, `Int`, `Float`, `String`, and their source-level methods.

**Repository reference:** [`phalcom-core/core/core.ph`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/core/core.ph#L1-L500)

The native methods, however, are installed through Rust registration calls such as `primitive!`, `primitive_static!`, and their internal/rest variants.

**Repository reference:** [`phalcom-core/src/universe/primitives.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/src/universe/primitives.rs#L1-L220)

A fully live editor surface therefore needs to eliminate the generated bridge rather than merely regenerate it more often.

### 2.7 AST already contains most required inference inputs

The AST has enough structure for meaningful dynamic inference without a language-level type system. It records:

- class declarations and superclass references;
- method/getter/setter/index bodies and parameters;
- imports and module bindings;
- mutable/immutable bindings;
- destructuring patterns;
- literals for integers, floats, strings, booleans, symbols and all collection families;
- variables, source/implementation fields, `self`, `super`;
- assignments;
- method/property/index sends;
- blocks and method references.

**Repository references:** [`phalcom-ast/src/ast.rs`, declarations](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-ast/src/ast.rs#L1-L420), [`phalcom-ast/src/ast.rs`, expressions/bindings](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-ast/src/ast.rs#L420-L860)

That makes local and interprocedural *runtime-shape inference* feasible. It does not make every value statically knowable: dynamic dispatch, reflective sends, opaque native primitives, runtime mutation, and intentionally late-bound code remain legitimate sources of `Unknown`.

---

## 3. VS Code extension state

### 3.1 LanguageClient wiring is already thin

`tools/vsphalcom/src/extension.ts` resolves a server path, starts `phalcom-lsp` over stdio, registers it for `file`/`phalcom` documents, and stops it on deactivation.

**Repository reference:** [`tools/vsphalcom/src/extension.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/extension.ts#L1-L80)

This is the correct end-state shape for a language-server extension. Standard LSP features such as inlay hints should remain server-owned; the extension should not reimplement them with `vscode.languages.registerInlayHintsProvider`.

### 3.2 Configuration is minimal

`package.json` currently exposes:

- `phalcom.executablePath`;
- `phalcom.lsp.enabled`;
- `phalcom.lsp.serverPath`.

It depends on `vscode-languageclient` and otherwise carries no inference/inlay settings.

**Repository reference:** [`tools/vsphalcom/package.json`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/package.json#L1-L120)

There is also no current configuration synchronization or initialization payload for semantic-engine settings, no language-server restart command, and no explicit bundled-binary resolution policy.

### 3.3 Extension testing is effectively absent

The extension test suite still contains the generated sample test (`Array.indexOf` assertions) rather than an LSP integration test.

**Repository reference:** [`tools/vsphalcom/src/test/suite/extension.test.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/test/suite/extension.test.ts#L1-L25)

By contrast, the Rust LSP already has in-process JSON-RPC integration tests for its original five stages.

**Repository references:** [`phalcom-lsp/tests/integration.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/tests/integration.rs#L1-L10), [`stage3_completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/tests/stage3_completion.rs#L1-L170)

The completion integration test proves the current `let m = Mover.new(); m.` path but does not exercise chains, parameters, fields, imported values, collection shapes, unions, or inlay hints.

### 3.4 User/developer documentation is stale or skeletal

The VS Code README currently says only that the extension is a WIP.

**Repository reference:** [`tools/vsphalcom/README.md`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/README.md#L1-L5)

`phalcom-lsp/src/lib.rs` also still describes the crate as “currently Stage 1,” even though the backend implements all five stages.

**Repository reference:** [`phalcom-lsp/src/lib.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/lib.rs#L1-L45)

The older U-LSP plan similarly opens by describing Stage 2 as underway. This documentation drift should be corrected as part of the implementation unit rather than left to accumulate.

---

## 4. Gap analysis

| ID | Gap | Current consequence | Priority |
|---|---|---|---|
| G-01 | No semantic database | Each feature performs ad-hoc resolution; no shared value knowledge | Critical |
| G-02 | No inlay-hint provider | VS Code cannot display inferred runtime-class hints | Critical |
| G-03 | Receiver is an identifier, not an expression | Chained sends, index/property receivers and parenthesized receivers lose completion | Critical |
| G-04 | Capitalized-call heuristic is unsound | Non-constructor class-side calls may be misclassified as instances | High |
| G-05 | Parameters have no call-site inference | `method(x) { x. }` cannot narrow from callers | High |
| G-06 | Method returns are not summarized | `factory.make().` cannot infer the result | Critical |
| G-07 | Fields are not modeled | `self.client.` / `_client.` cannot learn from constructor assignments | High |
| G-08 | No module graph/import resolution | Cross-file classes, module members, and inheritance cannot be followed | Critical |
| G-09 | Generated `core-table.json` | Builtin surfaces are build-time snapshots, not live semantic input | Critical |
| G-10 | Native surface lives only in Rust registration calls | LSP cannot soundly infer opaque primitive behavior without canonical metadata/contracts | High |
| G-11 | Unknown completion dumps merged builtin members | Suggestions are noisy and disconnected from live workspace evidence | Medium |
| G-12 | No semantic dependency invalidation | Interprocedural inference would otherwise require expensive whole-workspace recomputation | High |
| G-13 | Full-document sync + full parse per change | Acceptable today, but semantic analysis needs revisioned caching; later incremental parsing should remain possible | Medium |
| G-14 | Workspace scan is synchronous in `initialize` | Large workspaces can delay initialization | Medium |
| G-15 | No watched-file/workspace-folder semantic updates | Closed files changed externally can leave the index stale until restart/reopen | High |
| G-16 | VS Code tests do not test LSP behavior | Client/server integration and packaging regressions can ship unnoticed | High |
| G-17 | Server binary discovery depends on configured path/PATH | Extension is not self-contained for end users | High |
| G-18 | Documentation does not match implementation | Contributors are likely to design against obsolete stage state | Medium |

---

## 5. What “inference” can and cannot mean in Phalcom today

Phalcom is dynamically dispatched. The LSP therefore must not silently turn editor guesses into compile-time truth.

A useful editor fact is:

> “At this program point, the best currently known runtime value shape is `Point`, based on this initializer and these call/return facts.”

It is **not**:

> “The variable has the language type `Point` and values of other classes are illegal.”

This distinction matters because a mutable binding can be rebound, a method can return different classes on different paths, dispatch can be reflective, and a native primitive can hide its implementation from the VM-free analyzer.

The implementation should use a lattice with at least `Unknown`, one exact class/value shape, and bounded unions. Every result should carry confidence/provenance. Inlay hints should default to high-confidence results and explicitly describe themselves as inferred runtime information.

An opaque native primitive is the hard limit. If neither Phalcom source nor canonical native metadata says what it returns, the correct answer is `Unknown`. Executing arbitrary user/native code inside the language server to “find out” would be unsafe, nondeterministic, and semantically wrong.

---

## 6. Recommended target state

The target architecture is:

```text
VS Code
  │
  │ standard LSP
  ▼
phalcom-lsp Backend
  ├── DocumentStore        text + recovered AST + LineIndex + revision
  ├── SemanticDb
  │    ├── ModuleGraph
  │    ├── Symbol/Class Surface Index
  │    ├── Local Flow Facts
  │    ├── Field Facts
  │    ├── Callable Summaries
  │    ├── Reverse Dependencies
  │    └── Core/Native Source Model
  └── feature adapters
       ├── completion
       ├── inlay hints
       ├── hover
       ├── definition/references
       └── semantic tokens
```

Completion and inlay hints must query the same inferred facts. If hover later says a binding is `String`, completion for that binding must use the same `String` candidate set; there should be no independent resolver that can disagree.

---

## 7. Primary design conclusions

1. **Keep the LSP VM-free.** A live VM is not necessary for editor intelligence and would make completion dependent on executing user code.
2. **Do not build a hidden static type checker.** Build an advisory runtime-shape inference engine with `Unknown`, unions, confidence and provenance.
3. **Replace `ConstructResolver`; do not keep extending it.** Its trait was a good migration seam, but the receiver must become a real expression query into `SemanticDb`.
4. **Add a module graph before cross-file inference.** Bare class names are insufficient and the current URI-local class identity should not be weakened.
5. **Eliminate generated core JSON.** Parse core Phalcom source through the same semantic pipeline and establish one canonical source for native primitive declarations.
6. **Make native-return uncertainty explicit.** No source/metadata means `Unknown`; never invent a return class from naming conventions.
7. **Implement inlay hints as standard LSP.** The VS Code extension should remain a client shim.
8. **Add real VS Code end-to-end tests and binary packaging.** Protocol correctness alone is not complete integration.
9. **Use revisioned, dependency-aware semantic caching.** Full-file reparsing can remain initially, but semantic recomputation must be incremental by dependency.
10. **Treat future explicit typing as an input to this model.** If Phalcom later gains type annotations, they can become highest-priority facts without replacing the editor architecture.

---

## 8. Verification status

This report is based on direct inspection of repository content at the pinned commit above. The available environment did not provide an executable checkout of the repository, so this analysis did **not** rerun Cargo, npm, VS Code Electron, or language-corpus tests. The repository now exposes `scripts/test.sh lsp` as the consolidated LSP test lane; the implementation specification requires that lane plus extension E2E tests as acceptance gates.

**Repository reference:** [`scripts/test.sh`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/scripts/test.sh#L1-L80)

---

## 9. Key repository reference index

- [`phalcom-lsp/src/backend.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/backend.rs#L390-L500) — advertised LSP capabilities and document lifecycle.
- [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L45-L145) — `ReceiverResolver`, `ConstructResolver`, receiver-prefix extraction.
- [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L335-L445) — assignment heuristic and unknown fallback.
- [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L585-L705) — inheritance walk, builtin table, explicit no-import behavior.
- [`phalcom-lsp/src/index.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/index.rs#L150-L350) — module-local class map and per-file contributions.
- [`phalcom-lsp/src/core_table.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/core_table.rs#L1-L180) — embedded generated builtin table.
- [`phalcom-core/bin/gen-core-table/main.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/bin/gen-core-table/main.rs#L1-L260) — current generator.
- [`phalcom-core/src/universe/primitives.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/src/universe/primitives.rs#L1-L220) — native primitive registration source.
- [`phalcom-core/core/core.ph`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/core/core.ph#L1-L500) — real core Phalcom declarations.
- [`phalcom-ast/src/ast.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-ast/src/ast.rs#L1-L860) — semantic inputs available to a VM-free analyzer.
- [`tools/vsphalcom/src/extension.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/extension.ts#L1-L80) — current LanguageClient shim.
- [`tools/vsphalcom/package.json`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/package.json#L1-L120) — extension settings/dependencies.
- [`tools/vsphalcom/src/test/suite/extension.test.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/test/suite/extension.test.ts#L1-L25) — placeholder client test.
