# Phalcom Language Server + VS Code Integration Guide
## Target Developer and User Documentation

**Applies to:** the live semantic-intelligence design proposed for `phalcom-lsp` and `tools/vsphalcom`  
**Repository baseline:** `5b6d67be93d6167558931a5c5dae3ae69959c9c4`

---

# 1. Overview

Phalcom editor intelligence is provided by `phalcom-lsp`, a Rust language server launched by the VS Code extension over standard input/output. The extension itself should remain intentionally small: it discovers and starts the server, synchronizes configuration and workspace-file changes, exposes a few lifecycle commands, and lets standard LSP capabilities drive diagnostics, completion, hover, navigation, semantic highlighting, and inlay hints.

The language server does not run user programs and does not embed the Phalcom VM. It parses source with `phalcom-ast` and maintains a workspace semantic database.

The key conceptual rule is:

> **Editor “types” are inferred runtime value knowledge, not enforced Phalcom type annotations.**

A hint such as `: Point` means the analyzer has strong evidence that the value at that program point is a `Point`. It does not change the program and does not prevent the value from becoming another runtime class where Phalcom semantics permit that.

---

# 2. Current repository wiring

The current VS Code client already launches `phalcom-lsp` over stdio.

**Reference:** [`tools/vsphalcom/src/extension.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/extension.ts#L1-L80)

The server already provides diagnostics, completion, hover, definition/references, workspace symbols, and semantic tokens.

**Reference:** [`phalcom-lsp/src/backend.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/backend.rs#L390-L500)

The live-semantic implementation adds inference and standard LSP inlay hints without moving language analysis into TypeScript.

---

# 3. Target user experience

## 3.1 Literal inference

Source:

```phalcom
let name = "Phalcom"
name.
```

Editor:

```text
let name: String = "Phalcom"
          ^ inlay hint
```

Completion after `name.` is the live member surface of `String`.

## 3.2 Construction inference

```phalcom
let point = Point.new(10, 20)
point.
```

The server infers an instance of `Point` from resolved construction semantics, not merely because `Point` begins with an uppercase letter.

## 3.3 Chained return inference

```phalcom
factory.makePoint().
```

If the server can summarize `makePoint` as returning `Point`, completion after the trailing dot is the `Point` surface.

## 3.4 Parameter inference

```phalcom
render(_ shape) {
  shape.
}

render(Circle.new())
```

When dispatch is sufficiently resolved, the call site contributes `Circle` knowledge to `shape`. Multiple call sites may produce a union.

## 3.5 Field inference

```phalcom
class Service {
  @constructor
  new() {
    _client = HttpClient.new()
  }

  fetch() {
    _client.
  }
}
```

The field completion can use class-scoped assignment facts.

## 3.6 Uncertain values

If a value may be `Point` or `Vector`, the server can rank members shared by both classes first. A stable union may be displayed as:

```text
: Point | Vector
```

Heuristic-only guesses are hidden by default.

---

# 4. What the server intentionally does not do

The server does not execute code to infer values. It does not assume every capitalized call constructs an instance. It does not assign a return class to an opaque Rust primitive merely from a selector name.

When evidence is insufficient, the result is `Unknown`. Completion may still use live structural constraints and workspace surfaces, but the UI should not lie by displaying an exact type hint.

---

# 5. VS Code settings

Recommended final settings:

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

## `phalcom.lsp.enabled`

Enables the language server. When disabled, only non-LSP extension contributions such as the TextMate fallback grammar remain.

## `phalcom.lsp.serverPath`

Optional explicit path to `phalcom-lsp`.

When empty, the extension should prefer its bundled platform binary and use `$PATH` only as a development fallback.

## `phalcom.lsp.sysrootPath`

Optional path to the Phalcom core source/sysroot. This is mainly useful for language-runtime development or nonstandard installations.

## `phalcom.analysis.mode`

- `local`: analyze open/local source without workspace-wide call propagation.
- `workspace`: build the module/call graph and use cross-file evidence.

Default: `workspace`.

## `phalcom.inlayHints.types`

- `off`: no runtime-shape hints;
- `stable`: exact, flow and interprocedural facts;
- `all`: include heuristic structural guesses.

Default: `stable`.

## `phalcom.inlayHints.suppressObvious`

When true, the server may hide hints where the source itself is already visually obvious, such as a direct string literal.

## `phalcom.completion.unknownReceiver`

- `off`: no semantic fallback for a completely unknown receiver;
- `constrained`: use observed selectors/import scope to rank candidate classes;
- `workspace`: expose a broader live workspace selector surface.

Default: `constrained`.

## `phalcom.trace.server`

Controls LSP protocol logging in the language-client output channel. Recommended values: `off`, `messages`, `verbose`.

---

# 6. Extension lifecycle

Target activation flow:

```text
activate
  ├── register Run File command
  ├── resolve phalcom-lsp executable
  ├── read initialization settings
  ├── create .ph filesystem watcher
  ├── construct LanguageClient
  ├── start client
  ├── register Restart Language Server
  └── register Show Language Server Output
```

Target client options:

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

Settings that only change inference presentation should flow through LSP configuration without restarting the server. Changing the server executable path requires a controlled restart.

---

# 7. Server executable discovery

A production extension should not require users to manually install `phalcom-lsp` on `$PATH`.

Resolution order:

```text
configured serverPath
      ↓
bundled binary for current platform/architecture
      ↓
phalcom-lsp on PATH (development fallback)
```

The extension package should include server artifacts in predictable paths, for example:

```text
server/
  darwin-arm64/phalcom-lsp
  darwin-x64/phalcom-lsp
  linux-x64/phalcom-lsp
  linux-arm64/phalcom-lsp
  win32-x64/phalcom-lsp.exe
```

The exact supported platform matrix should match the Phalcom release pipeline.

If no executable can be started, report:

- the path attempted;
- the underlying spawn error;
- the `phalcom.lsp.serverPath` setting;
- a command to show the language-server output.

Do not silently fall back to no intelligence.

---

# 8. Workspace behavior

The server treats open buffers as authoritative.

```text
open unsaved buffer > disk version > missing file
```

This guarantees that adding a method in an unsaved file immediately changes completion.

Workspace changes that occur outside the editor must flow through file watchers. Added/removed workspace folders must update the module graph.

On closing a modified unsaved buffer, the semantic database should re-read the disk file rather than retaining the unsaved semantic contribution indefinitely.

---

# 9. Core source behavior

User/workspace source and core Phalcom source should enter the same semantic indexing pipeline.

The current generated core table is explicitly not the target architecture.

**Current static path:** [`core_table.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/core_table.rs#L1-L180)

**Current generator:** [`gen-core-table/main.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/bin/gen-core-table/main.rs#L1-L260)

**Core source:** [`core/core.ph`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/core/core.ph#L1-L500)

For runtime development, an edited workspace `core.ph` should become visible to completion without manually running a generator.

Native primitive selectors must also have a canonical declaration/descriptor shared with runtime installation. Native functions with no return contract remain `Unknown` to inference.

---

# 10. Inlay-hint semantics

Inlay hints are standard LSP `textDocument/inlayHint` responses.

A binding hint should be anchored after the bound identifier:

```text
let point: Point = makePoint()
```

Recommended confidence rendering:

| Confidence | Default | Rendering |
|---|---|---|
| exact | shown | `: Point` |
| local flow | shown | `: Point` |
| interprocedural | shown | `: Point` / `: Point | Vector` |
| heuristic | hidden | `≈ Drawable` when enabled |
| unknown | hidden | none |

Each nontrivial hint should provide a tooltip explaining its origin.

The hint is presentation only; copying the source from the editor should not insert the hint.

---

# 11. Completion semantics

Completion should use the inferred receiver expression, not a receiver-name regex.

Examples that must be supported:

```phalcom
variable.
self.member.
factory.make().
values[0].
(moduleFactory()).
```

For a single known class, complete that class's visible inherited surface.

For a bounded union, show common members first. Partial-union members should identify which candidate class provides them.

For an unknown receiver, use live workspace/core surfaces and structural evidence according to `phalcom.completion.unknownReceiver`.

The server should never need `core-table.json` to answer this.

---

# 12. Debugging

## 12.1 Language-server output

Use:

```text
Phalcom: Show Language Server Output
```

Set:

```json
"phalcom.trace.server": "messages"
```

or `verbose` for protocol-level debugging.

## 12.2 Server launch problems

Check:

1. `phalcom.lsp.enabled`;
2. `phalcom.lsp.serverPath`;
3. platform-bundled binary presence/permissions;
4. output-channel spawn errors.

## 12.3 Wrong completion

Useful diagnostic information to expose in debug logs:

```text
receiver source range
inferred ValueShape
confidence
fact provenance
resolved ClassId / ModuleId
visible member count
semantic generation
```

Do not log source contents by default.

## 12.4 Stale hints after cross-file edits

Verify:

- watched-file notification was received;
- target module revision changed;
- reverse dependency was invalidated;
- inlay-hint refresh was requested/handled.

---

# 13. Local development workflow

From repository root, build the LSP:

```sh
cargo build -p phalcom-lsp
```

Configure the Extension Development Host to use the local binary if the bundled dev path is not automatically selected:

```json
"phalcom.lsp.serverPath": "/absolute/path/to/target/debug/phalcom-lsp"
```

Build the extension:

```sh
npm --prefix tools/vsphalcom install
npm --prefix tools/vsphalcom run compile
```

Run focused language-server tests:

```sh
scripts/test.sh lsp
```

Run extension tests:

```sh
npm --prefix tools/vsphalcom test
```

The repository's consolidated LSP lane is defined in [`scripts/test.sh`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/scripts/test.sh#L1-L80).

---

# 14. Required VS Code E2E coverage

The current client test is only a sample assertion and must be replaced.

**Current reference:** [`extension.test.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/test/suite/extension.test.ts#L1-L25)

The E2E suite should open `.ph` fixtures and exercise VS Code's language-feature commands for:

- completion;
- inlay hints;
- hover;
- definition;
- diagnostics;
- configuration changes;
- server restart.

At least one test must edit a document without saving and verify that completion/inlay hints update from the live buffer.

At least one test must edit a second file and verify that a dependent open file receives updated cross-file inference.

---

# 15. Contributor guidance: adding semantic rules

Every new inference rule should answer five questions:

1. **What source evidence establishes the fact?**
2. **What confidence does the fact deserve?**
3. **What dependencies make it stale?**
4. **How does it join with conflicting evidence?**
5. **What dynamic operation forces it back toward `Unknown`?**

Avoid one-off code in completion or hover. Add the fact/query to `SemanticDb`, test it there, then consume it from feature adapters.

---

# 16. Contributor guidance: native primitives

Do not add new LSP-only native return maps.

If a primitive's selector or return contract is important to editor intelligence, add that information to the canonical source/native declaration mechanism that the runtime also consumes or validates.

The goal is not “keep the generated file synchronized.” The goal is to remove synchronization as a problem.

---

# 17. Future formal typing

If Phalcom later adds type annotations, the UI can evolve naturally:

```phalcom
let point: Point = makePoint()
```

At that point, explicit annotations become authoritative semantic facts. Inferred hints should normally disappear when the same information is written explicitly.

The semantic database remains useful for:

- inferred generic/element shapes;
- unannotated code;
- return inference;
- autocomplete;
- navigation;
- diagnostics against explicit declarations.

The current implementation should therefore avoid terminology and APIs that make `ValueShape` equivalent to a future language `Type`.

---

# 18. Repository files most relevant to implementation

- [`phalcom-lsp/src/backend.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/backend.rs#L390-L500)
- [`phalcom-lsp/src/documents.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/documents.rs#L1-L110)
- [`phalcom-lsp/src/index.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/index.rs#L150-L350)
- [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L45-L145)
- [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L335-L445)
- [`phalcom-lsp/src/completion.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/completion.rs#L585-L705)
- [`phalcom-lsp/src/core_table.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-lsp/src/core_table.rs#L1-L180)
- [`phalcom-core/core/core.ph`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/core/core.ph#L1-L500)
- [`phalcom-core/src/universe/primitives.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-core/src/universe/primitives.rs#L1-L220)
- [`phalcom-ast/src/ast.rs`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/phalcom-ast/src/ast.rs#L1-L860)
- [`tools/vsphalcom/src/extension.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/extension.ts#L1-L80)
- [`tools/vsphalcom/package.json`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/package.json#L1-L120)
- [`tools/vsphalcom/src/test/suite/extension.test.ts`](https://github.com/aureat/phalcom-lang/blob/5b6d67be93d6167558931a5c5dae3ae69959c9c4/tools/vsphalcom/src/test/suite/extension.test.ts#L1-L25)
