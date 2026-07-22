# Clickable Traceback Source Locations — Implementation Specification

**Status:** Proposed
**Scope:** Human and JSON runtime tracebacks
**Primary implementation area:** `phalcom-core/src/diagnostics`
**Compatibility target:** Existing traceback ordering, styling, frame elision, capture records, and source snippets remain intact.

---

## 1. Objective

Phalcom traceback source locations must be clickable in terminals and IDE-integrated terminals without making ordinary traceback output excessively long.

The implementation must separate three concepts:

1. **Canonical path** — the real absolute path identifying the source file.
2. **Display path** — the compact text shown to the user.
3. **Link target** — the URI embedded behind the display path when terminal hyperlinks are enabled.

A traceback may therefore display:

```text
main.ph:17:13
```

while linking it to:

```text
vscode://file/<canonical-path>:17:13
```

When no explicit hyperlink can safely be emitted, Phalcom must display a real, resolvable path such as:

```text
phalcom-core/tests/lang/runtime-errors/main.ph:17:13
```

Absolute paths are the last fallback, not the default presentation.

---

## 2. Non-goals

This unit does not implement:

* An editor protocol or language server.
* Repository-wide filesystem scanning.
* Detection of every terminal emulator.
* JetBrains-specific undocumented URI schemes.
* Source-map remapping for generated code.
* Clickable native frames.
* A change to traceback frame ordering or stack capture.
* Literal middle truncation such as `…/runtime/main.ph`.

A displayed path must either be a meaningful human label backed by an explicit hyperlink or a real path that terminal link detection can resolve.

---

## 3. Current-state diagnosis

The entry file is already canonicalized before module creation, and `ModuleObject` retains that absolute path.

The current traceback implementation loses this information:

* `FrameView` carries a module symbol, line, span, and source text, but no source path.
* Human rendering reconstructs the filename as `"{module}.ph"`.
* JSON rendering does the same.
* Captured `FrameRecord::Normal` values retain the module symbol and line, which is sufficient to recover the corresponding `ModuleObject` path at render time.
* Inline source uses the synthetic path `<main>` and must remain non-clickable.
* Native frames use `[native]` and must remain non-clickable.

The implementation must stop deriving filenames from module names.

---

## 4. User-facing configuration

Add two independent CLI options.

```text
--hyperlinks=auto|always|never
--trace-path=auto|short|relative|absolute
```

Defaults:

```text
--hyperlinks=auto
--trace-path=auto
```

### 4.1 Hyperlink modes

#### `auto`

Emit an explicit hyperlink only when:

* The output stream is a terminal.
* The terminal is not identified as `dumb`.
* Phalcom has positive evidence that an appropriate hyperlink target is supported.

Otherwise, emit plain text.

#### `always`

Emit OSC 8 links even when output is redirected or terminal support is unknown.

This mode is an explicit user override. Escape sequences in redirected output are acceptable under this mode.

For VS Code environments, use a `vscode://file` target. Otherwise use a `file://` target.

#### `never`

Never emit OSC 8 sequences.

This mode must not disable colors unless color configuration independently does so.

### 4.2 Trace path modes

#### `auto`

The recommended default.

* When an explicit hyperlink is active, display the shortest unique path suffix.
* When no explicit hyperlink is active, display a real project-relative path.
* For a file outside the diagnostic root, use its absolute path when no explicit hyperlink is active.

This mode adaptively chooses brevity or plain-text resolvability.

#### `short`

Always display the shortest unique path suffix, regardless of hyperlink support.

This mode maximizes brevity. Without an explicit hyperlink, the displayed suffix might not be directly resolvable from the working directory.

#### `relative`

Display the path relative to the diagnostic root whenever the file is beneath that root.

Use the absolute path for files outside the root.

#### `absolute`

Always display the canonical absolute path.

### 4.3 Orthogonality

The following options remain independent:

* `--color`
* `--plain`
* `--hyperlinks`
* `--trace-path`

In particular:

* `NO_COLOR` does not disable hyperlinks.
* `--plain` does not disable hyperlinks.
* `--color=never` does not disable hyperlinks.
* `--trace-format=json` always disables terminal styling and hyperlinks.

---

## 5. Diagnostic root

The **diagnostic root** is the canonical process working directory captured once during CLI startup.

It must not be recomputed during rendering.

```rust
let diagnostic_root = std::env::current_dir()
    .ok()
    .and_then(|path| path.canonicalize().ok());
```

Rules:

* Do not invoke Git.
* Do not search upward for `.git`.
* Do not infer the root from the entry file.
* Embedders and the REPL may provide an explicit root.
* If the root cannot be resolved, relative mode degrades to absolute paths.

This makes path rendering deterministic with respect to the invocation context.

---

## 6. Architecture

Add a dedicated source-location module:

```text
phalcom-core/src/diagnostics/location.rs
```

The module owns:

* Hyperlink policy.
* Terminal capability classification.
* Project-relative path calculation.
* Shortest-unique-suffix calculation.
* File URI generation.
* Visible location formatting.
* OSC 8 emission.
* Path sanitization.

It must not own:

* Color policy.
* Frame walking.
* Source span calculation.
* Error-message formatting.

### 6.1 Configuration types

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum HyperlinkMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum TracePathMode {
    Auto,
    Short,
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkFlavor {
    None,
    FileUri,
    VsCode,
}

#[derive(Clone, Debug)]
pub struct LocationConfig {
    pub hyperlinks: HyperlinkMode,
    pub trace_path: TracePathMode,
    pub link_flavor: LinkFlavor,
    pub diagnostic_root: Option<PathBuf>,
}
```

`HyperlinkMode::Auto` must be resolved once during startup into a concrete `LinkFlavor`.

No renderer may inspect environment variables directly.

### 6.2 Terminal capability resolution

Use a pure, unit-testable function:

```rust
pub fn resolve_link_flavor(
    mode: HyperlinkMode,
    stderr_is_terminal: bool,
    env: &TerminalEnvironment,
) -> LinkFlavor;
```

`TerminalEnvironment` is a captured subset of environment variables rather than direct global reads inside the function.

Minimum fields:

```rust
pub struct TerminalEnvironment {
    pub term: Option<String>,
    pub term_program: Option<String>,
    pub vscode_pid: Option<String>,
    pub vte_version: Option<String>,
    pub wt_session: Option<String>,
    pub konsole_version: Option<String>,
    pub kitty_window_id: Option<String>,
    pub wezterm_pane: Option<String>,
}
```

Resolution:

1. `never` → `None`.
2. `always`:

   * VS Code environment → `VsCode`.
   * Otherwise → `FileUri`.
3. `auto` and non-terminal output → `None`.
4. `auto` and `TERM=dumb` → `None`.
5. VS Code evidence → `VsCode`.
6. Known OSC 8 terminal evidence → `FileUri`.
7. Otherwise → `None`.

VS Code evidence:

* `TERM_PROGRAM=vscode`, or
* `VSCODE_PID` is present.

Conservative generic OSC 8 evidence:

* `TERM_PROGRAM=iTerm.app`
* `VTE_VERSION` present
* `WT_SESSION` present
* `KONSOLE_VERSION` present
* `KITTY_WINDOW_ID` present
* `WEZTERM_PANE` present

Unknown terminals remain plain-text by default. Users can override with `--hyperlinks=always`.

### 6.3 Configuration installation

Keep `RenderConfig` focused on color, glyphs, and width. Do not add an owned `PathBuf` to it or remove its `Copy` property.

Install location configuration separately:

```rust
static LOCATION_CONFIG: OnceLock<LocationConfig> = OnceLock::new();

pub fn install_location_config(config: LocationConfig);
pub fn active_location_config() -> &'static LocationConfig;
```

This mirrors the existing transitional render-configuration bridge.

Long term, both configurations may be passed explicitly through all diagnostic entry points. This unit does not need to complete that broader refactor.

---

## 7. Source path index

Build one source path index per rendered traceback.

```rust
pub struct SourcePathIndex {
    by_module: HashMap<Symbol, SourcePathRecord>,
}

pub struct SourcePathRecord {
    canonical: PathBuf,
    relative: Option<PathBuf>,
    shortest_unique: PathBuf,
}
```

Construction:

```rust
pub fn from_vm(vm: &VM, config: &LocationConfig) -> SourcePathIndex;
```

The index must inspect the paths of currently loaded modules in `VM::modules`.

It must not scan the repository filesystem.

### 7.1 Eligible source paths

A module path is eligible when it represents a real filesystem path.

Exclude:

* `<main>`
* `[native]`
* Empty paths
* Other bracketed or angle-bracketed synthetic identifiers
* Paths that cannot be interpreted as filesystem paths

Do not require the file still to exist at render time. A file may have been moved or deleted after its source was loaded, but its recorded identity remains useful.

### 7.2 Canonical path handling

Module paths are expected to have been canonicalized by the loader.

The traceback renderer must not call `canonicalize()` for every frame.

When a non-canonical path reaches the index:

* Resolve it against the diagnostic root when possible.
* Normalize `.` components lexically.
* Preserve unresolved `..` components rather than guessing.
* Never panic.

### 7.3 Relative path

When the canonical path is beneath the diagnostic root:

```rust
canonical.strip_prefix(diagnostic_root)
```

produces its relative path.

Otherwise, `relative` is `None`.

### 7.4 Shortest unique suffix

The shortest unique suffix is the smallest trailing sequence of path components that distinguishes one canonical source path from every other distinct eligible source path loaded in the VM.

Example:

```text
/project/examples/main.ph
/project/tests/main.ph
/project/lib/parser.ph
```

becomes:

```text
examples/main.ph
tests/main.ph
parser.ph
```

Algorithm:

1. Deduplicate identical canonical paths.
2. Start with one trailing component: the basename.
3. Compare against every other distinct canonical path.
4. If the suffix is not unique, include one additional parent component.
5. Continue until unique.
6. If every component is required, use the full relative path when available.
7. Otherwise use the canonical path.

Comparison must operate on path components, not string suffixes.

On Windows, comparison follows Windows case-insensitive path expectations. On Unix platforms, comparison is case-sensitive.

### 7.5 No literal ellipsis

Do not render:

```text
…/runtime-errors/main.ph
```

or:

```text
.../runtime-errors/main.ph
```

An ellipsis is neither a valid path component nor reliably recognized by IDE link parsers.

“Truncation” in this specification means omission of unnecessary leading components, not insertion of a marker.

---

## 8. Resolved source locations

Represent a concrete location independently of how it is rendered:

```rust
#[derive(Clone, Debug)]
pub struct ResolvedLocation {
    pub canonical_path: Option<PathBuf>,
    pub display_path: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}
```

Rules:

* Lines and columns are 1-based.
* `line == 0` becomes `None`.
* A column may exist only when a line exists.
* Captured records currently have no span and therefore no column.
* Synthetic locations have no canonical path and cannot become hyperlinks.

Visible formatting:

```text
path
path:line
path:line:column
```

No spaces occur between these components.

---

## 9. Frame data changes

Extend `FrameView` with an optional 1-based source column:

```rust
pub struct FrameView {
    pub module: Symbol,
    pub name: FrameName,
    pub line: u32,
    pub column: Option<u32>,
    pub span: SourceRange,
    pub source: Option<Arc<String>>,
    pub is_core: bool,
    pub fiber: u32,
    pub class_name: Option<Symbol>,
}
```

`StackWalk::build_view` computes the column from:

* The chunk-resolved source.
* The frame span’s starting byte offset.
* The existing shared `diagnostics::line_col` helper, or a moved equivalent.

Do not compute the column from UTF-8 byte distance alone.

### 9.1 Captured frames

Do not expand `FrameRecord::Normal` merely to add a column.

The existing compact shape remains:

```rust
Normal {
    module: Symbol,
    method: Symbol,
    line: u32,
}
```

When converting a captured record into a `FrameView`:

```rust
column: None
```

This preserves the existing compact capture contract.

---

## 10. Display path selection

The source path index resolves a display path using this matrix:

| Trace path mode |         Explicit hyperlink active |       Explicit hyperlink inactive |
| --------------- | --------------------------------: | --------------------------------: |
| `auto`          |            Shortest unique suffix | Relative path, otherwise absolute |
| `short`         |            Shortest unique suffix |            Shortest unique suffix |
| `relative`      | Relative path, otherwise absolute | Relative path, otherwise absolute |
| `absolute`      |                     Absolute path |                     Absolute path |

Synthetic locations ignore this matrix and retain their synthetic labels.

This is the core adaptive behavior.

---

## 11. Link targets

### 11.1 VS Code target

For `LinkFlavor::VsCode`, generate:

```text
vscode://file/<absolute-path>:<line>:<column>
```

Omit unavailable suffixes:

```text
vscode://file/<absolute-path>:<line>
vscode://file/<absolute-path>
```

The target must always use the canonical absolute path, never the display path.

### 11.2 Generic file target

For `LinkFlavor::FileUri`, generate a properly encoded `file://` URI.

Do not append editor-specific line fragments to a generic file URI. Generic line-fragment behavior is not portable.

The visible text still includes line and column information.

### 11.3 URI construction

Use a URL implementation capable of producing file URLs correctly across Unix and Windows.

Preferred implementation:

```rust
url::Url::from_file_path(...)
```

Add `url` as a workspace-pinned dependency if it is not already present.

Do not construct URIs through raw string concatenation.

Paths may contain:

* Spaces
* `#`
* `%`
* Non-ASCII characters
* Windows drive prefixes
* UNC prefixes

All must be encoded correctly.

### 11.4 OSC 8 encoding

Use the standard form with `ST`, not `BEL`:

```text
ESC ] 8 ; ; <URI> ESC \
<visible text>
ESC ] 8 ; ; ESC \
```

Do not leave a hyperlink open across:

* Whitespace outside the location
* `in <frame-name>`
* A newline
* Another traceback frame

Each location is one independently closed link.

---

## 12. Styling interaction

Hyperlinks and SGR styling are orthogonal.

The location renderer should conceptually produce:

```rust
osc8_open(uri)
+ styler.paint(Role::Location, visible_location)
+ osc8_close()
```

When links are disabled:

```rust
styler.paint(Role::Location, visible_location)
```

Requirements:

* `Styler` remains the only producer of SGR sequences.
* `location.rs` is the only producer of OSC 8 sequences.
* Color reset must occur before or within the hyperlink close boundary.
* Closing sequences must be emitted even when color is disabled.
* The visible bytes after removing SGR and OSC sequences must be identical to the plain render.

---

## 13. Renderer integration

Construct `SourcePathIndex` once at the beginning of traceback rendering.

Pass it through all human-rendering helpers.

Update these surfaces:

### 13.1 Ordinary frame lines

Replace:

```rust
let file = format!("{}.ph", module_name);
```

with resolution through `SourcePathIndex`.

Target output:

```text
  main.ph:17:13   in <main>
```

### 13.2 Captured frame lines

Resolve `FrameRecord::Normal.module` through the same index.

Captured and live frames must use identical path policy.

### 13.3 Fiber spawn locations

Where a fiber boundary identifies a real source module, resolve it through the same index.

If the existing `spawn_file` symbol contains a module symbol rather than a path symbol, document that invariant and preferably rename the field in a separate mechanical change:

```rust
spawn_module: Option<Symbol>
```

Do not guess whether an arbitrary interned string is a path.

### 13.4 Innermost caret block

The snippet header must use the same display path and hyperlink target as its corresponding frame.

A traceback must not show:

```text
main.ph:17
```

in the frame list and then:

```text
/Users/name/project/main.ph:17:13
```

in the caret header unless `--trace-path=absolute` was requested.

Extend the caret API to accept a structured location rather than only an untyped filename string.

Suggested compatible API:

```rust
impl Snippet {
    pub fn with_file(file: String) -> Self;
    pub fn with_location(location: ResolvedLocation) -> Self;
}
```

`with_file` remains available for existing parse-diagnostic callers during migration.

### 13.5 Native frames

`[native]` remains plain, non-clickable text.

### 13.6 Inline source

`<main>` remains plain, non-clickable text.

Do not generate a fake `main.ph` target for `-i` input.

---

## 14. JSON traceback behavior

JSON output must never contain:

* SGR sequences
* OSC 8 sequences
* Display-only truncation

For real source files, the JSON frame’s `file` field must contain the canonical stored module path rather than a reconstructed `module.ph`.

Recommended frame shape:

```json
{
  "module": "main",
  "file": "/canonical/project/main.ph",
  "line": 17,
  "column": 13,
  "name": "<main>",
  "core": false,
  "fiber": 1
}
```

For captured frames without a column:

```json
"column": null
```

For synthetic frames:

```json
"file": "[native]"
```

or:

```json
"file": "<main>"
```

Path-dependent tests must normalize the repository prefix to a placeholder such as `<ROOT>` before comparison.

This is an intentional correction to the JSON frame identity contract.

---

## 15. Security and malformed paths

Filesystem paths and module metadata must not be able to inject terminal control sequences.

Before producing visible text:

* Replace ESC, BEL, carriage return, newline, and other C0 control characters with escaped or replacement forms.
* Never copy control bytes directly into terminal output.
* Preserve ordinary Unicode.
* Do not interpret a source path beginning with a URI scheme as a prebuilt target.
* Construct link targets only from filesystem-path APIs.

Before emitting an OSC 8 link:

* Require a canonical filesystem path.
* Require a successfully constructed URI.
* On failure, render plain visible text.
* Never omit the closing OSC sequence after emitting an opening sequence.

---

## 16. Required tests

### 16.1 Shortest-unique-suffix unit tests

Cover:

```text
/a/main.ph
/b/main.ph
```

Expected:

```text
a/main.ph
b/main.ph
```

Cover:

```text
/a/runtime/main.ph
/b/runtime/main.ph
```

Expected:

```text
a/runtime/main.ph
b/runtime/main.ph
```

Cover:

```text
/a/parser.ph
/b/runtime.ph
```

Expected:

```text
parser.ph
runtime.ph
```

Also cover:

* The same canonical file registered under multiple modules.
* One path being a suffix of another.
* Root-level files.
* Unicode path components.
* Windows drive paths under `cfg(windows)`.
* Case-only differences according to platform behavior.

### 16.2 Display-mode tests

For every `TracePathMode`, test with:

* Hyperlinks active.
* Hyperlinks inactive.
* File under diagnostic root.
* File outside diagnostic root.
* Missing diagnostic root.
* Synthetic source.

### 16.3 Link-resolution tests

Test:

* `never` always resolves to `None`.
* `always` in VS Code resolves to `VsCode`.
* `always` elsewhere resolves to `FileUri`.
* `auto` with redirected stderr resolves to `None`.
* `auto` with `TERM=dumb` resolves to `None`.
* VS Code detection.
* Every supported generic OSC 8 environment marker.
* Unknown TTY resolves to `None`.

Tests must pass an explicit environment structure. They must not mutate global process environment in parallel tests.

### 16.4 URI tests

Cover filenames containing:

```text
space name.ph
hash#name.ph
percent%name.ph
ünicode.ph
```

On Windows also cover:

```text
C:\Project Dir\main.ph
```

Assert that raw spaces, `#`, and control characters do not appear unencoded in the URI target.

### 16.5 Human-render tests

Required cases:

1. Single uniquely named file, hyperlink active:

   ```text
   main.ph:17:13
   ```

2. Duplicate basename, hyperlink active:

   ```text
   runtime-errors/main.ph:17
   examples/main.ph:9
   ```

3. Unknown terminal, auto mode:

   ```text
   phalcom-core/tests/lang/runtime-errors/main.ph:17:13
   ```

4. File outside diagnostic root, no hyperlink:

   ```text
   /absolute/shared/main.ph:17
   ```

5. Captured frame:

   ```text
   main.ph:17
   ```

6. Inline source:

   ```text
   <main>:1
   ```

   with no OSC 8 sequence.

7. Native frame:

   ```text
   [native]
   ```

   with no OSC 8 sequence.

8. Color plus hyperlink:

   * SGR and OSC sequences are correctly nested.
   * Neither style nor link leaks into `in <name>`.

9. Redirected auto output:

   * No OSC sequences.
   * No SGR sequences unless `--color=always`.
   * Plain path remains resolvable.

### 16.6 Terminal-control stripping helper

Add a test-only helper that strips both:

* SGR sequences.
* OSC 8 open and close sequences.

For every linked rendering test:

```rust
assert_eq!(
    strip_terminal_controls(&linked),
    plain
);
```

This proves that hyperlinks do not change visible traceback content.

### 16.7 JSON tests

Assert:

* Canonical path is present for real files.
* No OSC byte exists.
* No SGR byte exists.
* Column is present for live frames.
* Column is null for compact captured frames.
* Synthetic paths remain synthetic.

---

## 17. Integration examples

### 17.1 VS Code terminal

Visible:

```text
Traceback (most recent call last):
  main.ph:17:13   in <main>
      let d = Derived.new()
  derived.ph:5:9   in new()
      class.bump()
× None does not understand '+(_)'
```

The visible location `main.ph:17:13` links to its canonical `vscode://file` target.

### 17.2 Unknown terminal

```text
Traceback (most recent call last):
  examples/main.ph:17:13   in <main>
  src/models/derived.ph:5:9   in new()
× None does not understand '+(_)'
```

The locations are plain text and correspond to real paths relative to the invocation directory.

### 17.3 Duplicate basenames with links

```text
Traceback (most recent call last):
  examples/main.ph:17:13   in <main>
  runtime-errors/main.ph:5:9   in new()
```

Both compact labels have distinct canonical targets.

---

## 18. Implementation sequence

### Phase 1 — Location primitives

Add:

* `HyperlinkMode`
* `TracePathMode`
* `LinkFlavor`
* `LocationConfig`
* `ResolvedLocation`
* URI and OSC 8 helpers
* Unit tests

No traceback behavior changes yet.

### Phase 2 — CLI configuration

Add both flags to `Cli`.

Resolve:

* Terminal status.
* Environment capabilities.
* Diagnostic root.

Install `LocationConfig` once at startup.

### Phase 3 — Source path index

Implement `SourcePathIndex::from_vm`.

Add shortest-unique-suffix and path-selection tests.

### Phase 4 — Frame column

Add `FrameView::column`.

Compute it in the live stack walk.

Captured records continue to set it to `None`.

### Phase 5 — Human traceback integration

Replace every reconstructed `module.ph` location in:

* Live frames.
* Captured frames.
* Fiber boundary locations.
* Innermost snippet header.

Preserve all existing ordering, elision, source echo, and styling behavior.

### Phase 6 — JSON correction

Use canonical source paths and optional columns.

Normalize repository prefixes in tests.

### Phase 7 — Documentation and verification

Update:

* CLI help.
* Traceback implementation specification.
* Output catalog.
* Human-render canary snapshots.
* JSON structural fixtures.

Run the full verification gate.

---

## 19. Acceptance criteria

The implementation is complete when all of the following hold:

1. A real source file’s canonical path is never reconstructed from its module name.
2. Default VS Code output displays compact source labels that open the exact file and location.
3. Default unknown-terminal output contains plain, resolvable paths.
4. Duplicate basenames expand only as far as needed when compact links are active.
5. No repository-wide scan occurs.
6. Redirected default output contains no OSC 8 sequences.
7. JSON output contains no terminal escapes.
8. Inline and native frames never receive fake links.
9. Caught and live errors use the same path-selection policy.
10. Absolute paths appear by default only when required for plain-text resolvability.
11. `--trace-path=absolute` and `--hyperlinks=never` provide deterministic debugging overrides.
12. Existing traceback ordering, frame budgeting, core elision, source echoes, and error messages remain unchanged.

---

## 20. Final ruling

Phalcom’s default traceback location policy is:

> Use the shortest unique visible path when a canonical hidden target is available; otherwise print a real path that the terminal can resolve.

Canonical identity, visible brevity, and terminal navigation are separate responsibilities and must remain separate in the implementation.
