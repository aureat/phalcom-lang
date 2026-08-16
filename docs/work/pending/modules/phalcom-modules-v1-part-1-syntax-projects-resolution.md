# Phalcom Modules v1 Implementation Specification — Part I
## Surface syntax, project/package identity, logical resolution, path visibility, source ownership, and module interfaces

**Status:** Implementation specification
**Target:** Phalcom first-version static module/package/project system
**Repository:** `aureat/phalcom-lang`
**Repository snapshot inspected:** `ed841918546610752ec0b1d3f7b1ffa6b2056006` (`main`)
**Companion documents:** Part II — static linking/compiler/LSP; Part III — runtime/initialization/execution
**Design basis:** `modules-next.md` plus the corrective analysis in `modules-analysis.md`, amended by the subsequently ratified decisions described below.

---

# 1. Purpose and implementation boundary

This part establishes the compile-time ownership and resolution substrate on which the rest of the module system depends. It intentionally does **not** implement runtime module loading, module initialization, or bytecode import execution. Those are addressed in Parts II and III.

After this part lands, the toolchain must be able to answer, without executing Phalcom code:

1. Which project owns a source file?
2. What is the source unit's canonical `ModuleId`?
3. Is the unit an ordinary module or a package?
4. Which logical import does a source import denote?
5. Is that path externally addressable from the importing project?
6. Which bindings does a module declare and export?
7. Which bindings are imported locally?
8. Which static dependency edges exist?
9. Which package children are externally exposed?
10. Which module/package metadata attributes are attached as inert metadata?
11. Which source location corresponds to a semantic module identity?
12. Which errors can be rejected before bytecode generation?

The central architectural rule is:

```text
physical source location
        ↓ SourceProvider
SourceId / SourceLocation
        ↓ project ownership
ResolvedProjectId + project-relative module path
        ↓
ModuleId
        ↓
ModuleInterface
```

No runtime `Module` object participates in any of these answers.

---

# 2. Ratified language decisions implemented by this part

The following decisions are normative for this implementation.

1. Source imports name **logical modules**, never filesystem paths.
2. There is no `sys.path`-like or current-directory import search.
3. Packages are explicit and require `package.ph`.
4. A project owns one root package and defines its stable namespace through `project.toml`.
5. The checkout directory name is not part of project-backed module identity.
6. Project dependency aliases are resolver roots, not canonical identities.
7. Binding visibility and module-path visibility are independent.
8. Module bindings are private unless explicitly exported.
9. Child module/package paths are **project-private by default**.
10. A dependency's root package is externally addressable by its dependency alias.
11. Every deeper cross-project path component must be explicitly exposed by its immediate parent package.
12. `expose .child` is a static resolver declaration only. It does not import, initialize, bind, or export a runtime value.
13. Deep exposure is hierarchical; a package can expose only its own immediate children.
14. Static imports form a contiguous dependency preamble.
15. Import ordering inside the preamble has no runtime meaning.
16. Import bindings have module-wide semantic scope after successful linking.
17. Whole-module import binds the final path component unless an explicit alias is given.
18. Selective imports support aliases and parenthesized grouping.
19. Exports are explicit; imported names are not automatically re-exported.
20. Direct re-export syntax `export X from .m` is supported in v1.
21. Parenthesized grouped exports/re-exports are supported.
22. Wildcard import/export does not exist.
23. `from package import X` means an exported binding named `X`; it never falls back to a child module lookup.
24. A package façade can re-export values from private implementation modules without exposing those module paths.
25. Module/package attributes in this version are **metadata-only**. They cannot execute, expand code, mutate the graph, replace objects, intercept dispatch, or generate initialization.
26. Runtime/compile-time transforming module/package attributes are deferred.
27. Runtime/dynamic/lazy loading is deferred and receives no syntax in this implementation.
28. Semantic/interface graph architecture must admit cycles even though runtime initialization cycles will be rejected in Part II.
29. Package containment does not imply ancestor-package initialization; this part therefore records containment without creating runtime edges.
30. The core/root namespace is explicit in resolver data; no import meaning depends on CWD or environment-variable search paths.

---

# 3. Current repository state that must be replaced

The current repository is still the U15 file-relative import implementation. The following exact anchors are the migration points.

## 3.1 AST

File: `phalcom-ast/src/ast.rs`

Current anchors:

- `Program { statements: Vec<Statement> }`
- `Statement::Import(ImportStatement)`
- `ImportStatement`
- `Module`
- `Attribute`
- `ClassDef::attributes`

Current `ImportStatement` represents the old shape:

```text
import "path" as Name
```

with a string path and a mandatory binding. That representation cannot express:

- logical absolute paths;
- relative dotted paths;
- selective imports;
- grouped imports;
- direct re-exports;
- path exposure;
- dependency-preamble structure.

Do not extend the old `path: String` structure with flags. Replace it.

## 3.2 Lexer/token layer

Files:

- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs`

Current token set includes `Import` and `As`, but no dedicated `From`, `Export`, or `Expose` token. The scanner is hand-written; keyword recognition is centralized in `Lexer::scan_identifier`.

## 3.3 Parser

File: `phalcom-ast/src/parser.rs`

Current anchors:

- `Parser::parse_program`
- statement dispatch arm `Token::Import => self.parse_import()`
- `Parser::parse_import`

`parse_import` currently requires a `Token::String` path and `as`. Replace the production rather than adding a second legacy branch. Physical import strings are removed language semantics, not a compatibility mode.

## 3.4 Compiler/runtime coupling

Files:

- `phalcom-core/src/compiler/lib/mod.rs::compile_import`
- `phalcom-core/src/bytecode.rs::Bytecode::Import`
- `phalcom-core/src/vm/dispatch.rs` `Bytecode::Import` arm
- `phalcom-core/src/interpret.rs::resolve_import_path`
- `phalcom-core/src/interpret.rs::VM::import_module`
- `phalcom-core/src/universe/mod.rs::Universe::module_registry`

These are deliberately not rewritten in Part I. Part I creates the semantic replacement they will consume; Parts II–III delete these runtime-path mechanisms.

## 3.5 LSP duplication

Files:

- `phalcom-lsp/src/semantic/ids.rs`
- `phalcom-lsp/src/semantic/module_graph.rs`

Current LSP `ModuleId` is a URI string, and the LSP independently resolves `import "./x"` through filesystem paths. This is exactly the duplication the new shared module subsystem must remove.

---

# 4. Add a VM-free `phalcom-modules` crate

Create a new workspace crate:

```text
phalcom-modules/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs
    ├── identity.rs
    ├── manifest.rs
    ├── project.rs
    ├── source.rs
    ├── resolver.rs
    ├── interface.rs
    ├── graph.rs
    └── metadata.rs
```

Modify root `Cargo.toml` workspace members to add:

```toml
"phalcom-modules",
```

The dependency direction must be:

```text
phalcom-common
      ↑
phalcom-ast
      ↑
phalcom-modules
    ↗        ↖
phalcom-core phalcom-lsp
```

`phalcom-modules` may depend on `phalcom-ast` because interface construction consumes parsed programs. It must not depend on `phalcom-core`, `VM`, heap objects, `ObjRef`, or `tower-lsp`.

Recommended `phalcom-modules/Cargo.toml`:

```toml
[package]
name = "phalcom-modules"
version = "0.1.0"
edition = "2024"

[dependencies]
phalcom-ast = { path = "../phalcom-ast" }
phalcom-common = { path = "../phalcom-common" }
thiserror = { workspace = true }
serde = { version = "1", features = ["derive"] }
toml = "0.9"
```

If the workspace has standardized `serde`/`toml` versions by implementation time, use workspace dependencies instead. Do not make the LSP depend on `phalcom-core`; its current VM-free boundary is correct and must remain intact.

`phalcom-common` should remain small. Semantic module/project resolution is substantial enough to justify its own crate rather than turning `phalcom-common` into a filesystem/project-management crate.

---

# 5. Semantic identity model

Implement the identity types in `phalcom-modules/src/identity.rs`.

## 5.1 `ResolvedProjectId`

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolvedProjectId(u32);
```

This is an **opaque graph-node identity**, not a semantic string such as `"foo@1.2.3"`.

Rules:

- Two dependency aliases resolving to one resolved project graph node share the same `ResolvedProjectId`.
- Two distinct resolved graph nodes have distinct identities even if their project names, namespace names, versions, and relative module paths are textually equal.
- Source-level dependency aliases never enter this identity.
- The numeric payload is not serialized into user artifacts as a stable public identifier.
- The project universe owns issuance and interning.

The project node should retain human-readable metadata separately:

```rust
pub struct ResolvedProject {
    pub id: ResolvedProjectId,
    pub name: String,
    pub namespace: ModuleComponent,
    pub source_root: SourceRoot,
    pub entry: Option<ModulePath>,
    pub dependencies: BTreeMap<ModuleComponent, ResolvedProjectId>,
    pub source_identity: ProjectSourceIdentity,
}
```

## 5.2 `ModuleComponent`

Use a validated newtype rather than passing arbitrary `String`s:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleComponent(Box<str>);
```

Construction must validate Phalcom identifier spelling once.

The resolver must never repeatedly revalidate components on hot paths.

## 5.3 `ModulePath`

A project-relative module path:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModulePath(Box<[ModuleComponent]>);
```

The root package uses the empty relative path:

```text
project(namespace = geometry)
relative path = []
display name = geometry
```

`geometry.shapes.circle` becomes:

```text
ResolvedProjectId(…)
ModulePath(["shapes", "circle"])
```

This choice is important: the project's own declared namespace is presentation/resolution metadata, not duplicated into every canonical relative path.

Provide:

```rust
impl ModulePath {
    pub fn root() -> Self;
    pub fn parent(&self) -> Option<Self>;
    pub fn join(&self, component: ModuleComponent) -> Self;
    pub fn components(&self) -> &[ModuleComponent];
    pub fn is_root(&self) -> bool;
}
```

## 5.4 `ModuleId`

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId {
    pub project: ResolvedProjectId,
    pub path: ModulePath,
}
```

This becomes the canonical toolchain identity.

Do **not** use:

- absolute path;
- URI;
- dependency alias;
- project namespace string alone;
- runtime `ObjRef`;
- process-global incrementing `u32` from `heap/module.rs`.

The existing `phalcom-core/src/heap/module.rs::ModuleId = u32` and `next_module_id()` are retired in Part III.

## 5.5 `SourceId` and `SourceLocation`

Keep source identity distinct:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(Box<str>);

#[derive(Clone, Debug)]
pub struct SourceLocation {
    pub source_id: SourceId,
    pub display_path: PathBuf,
}
```

A filesystem provider can use canonical-path-derived `SourceId`s internally. A future archive/cache/generated-source provider can use a different scheme without changing `ModuleId`.

This implements the analysis document's key deduction:

```text
ModuleId      = semantic ownership identity
SourceId      = source-provider identity
SourceLocation = diagnostics/UI location
```

Do not make `ModuleId` a disguised canonical filesystem path.

---

# 6. Project manifests

Implement `phalcom-modules/src/manifest.rs`.

## 6.1 Manifest schema

Parse:

```toml
[project]
name = "geometry-kit"
namespace = "geometry"
source = "src"
entry = "geometry.main"

[dependencies]
math = { package = "phalcom-math", version = "2.0" }
util = { path = "../utility" }
```

Types:

```rust
#[derive(Debug, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    pub namespace: String,
    #[serde(default = "default_source_root")]
    pub source: PathBuf,
    pub entry: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Path { path: PathBuf },
    Package { package: String, version: String },
}
```

Validation is a second step after TOML decoding. Do not combine syntax errors and semantic manifest errors.

Validate:

- `name` non-empty;
- `namespace` valid `ModuleComponent`;
- dependency aliases valid `ModuleComponent`;
- alias != self namespace;
- aliases unique by TOML map construction;
- aliases do not collide with reserved roots such as `core`;
- source root exists and is a directory when loading a runnable project;
- source root contains `package.ph`;
- entry, when present, is an absolute logical module path rooted at the project's own namespace;
- entry resolves to a module belonging to this project;
- entry is not a dependency path;
- path dependency points to another project containing `project.toml`;
- nested path dependencies get their own `ResolvedProjectId`.

## 6.2 Dependency-resolution boundary

Package acquisition/version solving is not the module resolver's job.

Define:

```rust
pub trait DependencyProvider {
    fn resolve_package(
        &self,
        package: &str,
        version_requirement: &str,
    ) -> Result<ResolvedDependencySource, ProjectError>;
}
```

The module subsystem consumes resolved dependency source locations. Path dependencies can be resolved directly in v1.

If no package-manager provider is configured and a registry/package dependency appears, emit a dedicated `UnresolvedPackageDependency` diagnostic. Do not search installation directories.

This keeps the module implementation complete without smuggling in a package manager.

---

# 7. `ProjectUniverse`

Implement `phalcom-modules/src/project.rs`.

```rust
pub struct ProjectUniverse {
    projects: Vec<ResolvedProject>,
    roots: BTreeMap<ProjectSourceIdentity, ResolvedProjectId>,
}
```

Responsibilities:

1. Parse and validate root `project.toml`.
2. Resolve path/package dependency source roots.
3. Detect project dependency cycles.
4. Assign one `ResolvedProjectId` per resolved graph node.
5. Build each project's import-root table:
   - self namespace -> self project id;
   - dependency alias -> dependency project id;
   - reserved roots -> reserved project/provider ids.
6. Detect root-name collisions before any source module resolution.
7. Own source-root confinement metadata.
8. Produce deterministic diagnostics.

The **project dependency graph must be acyclic**. This is separate from module semantic cycles.

Use DFS colors or Kahn's algorithm and report a concrete alias/project cycle:

```text
ProjectDependencyCycleError:
  app
  → util
  → logging
  → app
```

Do not defer this to module traversal.

---

# 8. Project ownership and nested projects

A physical file may be under one source root but inside a nested project checkout. The resolver must not accidentally assign it to both projects.

Normative rule:

> A nested directory containing `project.toml` is a project ownership boundary. Its content does not participate in an enclosing project's source namespace unless it is explicitly resolved as a dependency project.

For CLI source selection, nearest owning project wins.

Implement a helper in `project.rs`:

```rust
pub fn discover_owning_project(source: &Path) -> Result<Option<ProjectRoot>, ProjectError>;
```

Search upward from the selected source/directory for the nearest `project.toml`, subject to filesystem root termination. This is tooling discovery only; it does not become import search behavior.

---

# 9. Source-provider architecture

Implement `phalcom-modules/src/source.rs`.

## 9.1 Trait

```rust
pub trait SourceProvider {
    fn locate(
        &self,
        project: &ResolvedProject,
        path: &ModulePath,
    ) -> Result<SourceUnit, ModuleResolutionError>;

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError>;
}
```

`SourceUnit`:

```rust
pub struct SourceUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
}
```

`ModuleKind`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleKind {
    Module,
    Package,
}
```

## 9.2 Filesystem provider

Create:

```rust
pub struct FilesystemSourceProvider {
    // Canonicalized roots and directory-shape cache.
}
```

Resolution for a project-relative path `["shapes", "circle"]`:

1. Begin at the project's configured source root.
2. For every intermediate component, require `<component>/package.ph`.
3. Final component must resolve to exactly one of:
   - `circle.ph` => ordinary module;
   - `circle/package.ph` => package.
4. If both exist, `AmbiguousModuleError`.
5. Canonicalize selected source for confinement and duplicate-source validation.
6. Ensure canonical source remains under the project's canonical source root.
7. Never traverse a directory that is a nested project ownership boundary.
8. Return semantic `ModuleId` independently of the canonical path.

Root package path `[]` maps to `<source-root>/package.ph`.

## 9.3 Caching

Do not `canonicalize()` every path component on every import. Cache directory/source resolution by:

```rust
(ProjectId, ModulePath) -> Result<SourceUnit, CachedResolutionError>
```

and canonical source identity by physical path.

The common-case import after the first probe should be an in-memory hash/tree lookup.

Avoid scanning the entire source root merely to compile a small reachable graph. LSP workspace indexing may choose to enumerate files separately; the compiler resolver should remain demand-driven.

---

# 10. Package semantics

A package directory is recognized only by `package.ph`.

Examples:

```text
src/
├── package.ph              # root package
├── point.ph                # child module
└── shapes/
    ├── package.ph          # child package
    └── circle.ph
```

Canonical identities:

```text
geometry
geometry.point
geometry.shapes
geometry.shapes.circle
```

`package.ph` is the source of `geometry.shapes`, not `geometry.shapes.package`.

A directory without `package.ph` cannot occur as a traversed logical path.

The collision:

```text
network.ph
network/package.ph
```

is always an error. Never choose by lookup precedence.

Package containment recorded here is a resolution relation only:

```text
geometry contains shapes
```

It does **not** create:

```text
initialize geometry before geometry.shapes
```

Part III implements the ratified no-ancestor-initialization rule.

---

# 11. Standalone ownership modes

The resolver supports three execution ownership modes.

## 11.1 Project-backed source

If the selected source is inside a recognized project source root, project identity wins.

Stable under checkout relocation:

```text
/tmp/a/src/point.ph
/home/x/checkout/src/point.ph
```

both become the same logical project-relative module when loaded as the same resolved project.

## 11.2 Standalone package

A contiguous hierarchy of directories containing `package.ph` may form a package without `project.toml`.

Root package name is the outermost contiguous package directory's valid identifier.

Example:

```text
demo/
├── package.ph
└── tools/
    ├── package.ph
    └── inspect.ph
```

identities:

```text
demo
demo.tools
demo.tools.inspect
```

Standalone package identity is intentionally sensitive to directory/package-boundary renames. Document this explicitly.

## 11.3 Standalone module

A `.ph` file outside any project/package hierarchy can run as a standalone module.

It gets:

- one execution-local module identity;
- core visibility;
- reserved built-in roots only.

It does **not** gain sibling imports. `import helper` never means “find `helper.ph` next to this file.”

---

# 12. Logical import path syntax

Replace physical path strings with an explicit AST.

In `phalcom-ast/src/ast.rs` add:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ImportPath {
    pub root: ImportRoot,
    pub segments: Vec<PathSegment>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportRoot {
    Absolute(PathSegment),
    Relative { dots: u16, range: SourceRange },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathSegment {
    pub name: String,
    pub range: SourceRange,
}
```

Relative semantics:

- one leading dot = current package context;
- each additional dot ascends one parent package.

From module `app.geometry.circle`:

```phalcom
.point      // app.geometry.point
..model     // app.model
```

From package module `app.geometry`:

```phalcom
.point      // app.geometry.point
```

Relative imports cannot ascend above the root package.

Absolute roots are only:

- current project's own namespace;
- dependency aliases;
- reserved roots.

There is no implicit relative interpretation of a bare root.

---

# 13. New import/export/expose AST

Replace `ImportStatement` with declarations capable of expressing the full surface.

Recommended AST:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ModulePreamble {
    pub metadata: Vec<ModuleMetadataAttribute>,
    pub dependencies: Vec<DependencyDecl>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DependencyDecl {
    Import(ImportDecl),
    ReExport(ReExportDecl),
    Expose(ExposeDecl),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportDecl {
    Module(ModuleImportDecl),
    Selective(SelectiveImportDecl),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleImportDecl {
    pub path: ImportPath,
    pub alias: Option<ImportAlias>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectiveImportDecl {
    pub path: ImportPath,
    pub items: Vec<ImportItem>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportItem {
    pub name: String,
    pub name_range: SourceRange,
    pub alias: Option<ImportAlias>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReExportDecl {
    pub path: ImportPath,
    pub items: Vec<ExportItem>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportItem {
    pub local_or_remote_name: String,
    pub name_range: SourceRange,
    pub alias: Option<ExportAlias>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExposeDecl {
    pub child: PathSegment,
    pub range: SourceRange,
}
```

Change `Program` to make the dependency preamble structural:

```rust
pub struct Program {
    pub preamble: ModulePreamble,
    pub statements: Vec<Statement>,
}
```

Local export declarations that refer to body bindings remain body-level static statements/declarations:

```rust
Statement::Export(ExportDecl)
```

where:

```rust
pub struct ExportDecl {
    pub items: Vec<ExportItem>,
    pub range: SourceRange,
}
```

This split gives the compiler a direct guarantee:

```text
Program.preamble = every dependency-affecting declaration
Program.statements = executable/declaration body + local exports
```

A direct re-export belongs in the preamble because it creates a dependency without requiring a preceding body declaration.

---

# 14. Surface grammar

Add contextual/fixed keywords as appropriate:

- `from`
- `export`
- `expose`

Prefer dedicated `Token` variants for these language-level statement introducers, matching existing `Token::Import`/`Token::As`.

Update:

- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs::scan_identifier`
- `phalcom-ast/tests/lexer.rs`
- token snapshots.

## 14.1 Whole-module import

```phalcom
import geometry.point
import geometry.point as pointModule
import .point
import ..units as units
```

Default local binding is the final component.

Examples:

```phalcom
import geometry.shapes.circle   // binds `circle`
import geometry                // binds `geometry`
import .point                   // binds `point`
```

## 14.2 Selective import

Flat:

```phalcom
from geometry.point import Point, distance
from geometry.point import Point as P, distance as dist
```

Grouped:

```phalcom
from geometry.point import (
    Point,
    distance,
    origin as defaultOrigin,
)
```

Trailing comma in grouped form is allowed.

## 14.3 Local export

Flat:

```phalcom
export Point, Vector
export CartesianPoint as Point
```

Grouped:

```phalcom
export (
    Point,
    Vector,
    distance,
)
```

Local export does not create another local binding for an alias.

## 14.4 Direct re-export

Flat:

```phalcom
export Point from .point
export CartesianPoint as Point from .cartesian
```

Grouped:

```phalcom
export (
    Point,
    origin,
    distance as pointDistance,
) from .point
```

Normative lowering:

```phalcom
export Point from .point
```

is semantically equivalent to:

```phalcom
from .point import Point
export Point
```

including creation of an immutable local import binding. The linker may optimize away redundant local storage, but semantic name availability is the same.

## 14.5 Path exposure

Only in `package.ph`:

```phalcom
expose .point
expose .shapes
```

The operand must denote exactly one immediate child.

Reject:

```phalcom
expose .shapes.circle
expose ..foo
expose geometry.point
```

The parent package owns only its immediate public path surface.

---

# 15. Dependency-preamble grammar and source ordering

`Parser::parse_program` must parse in phases:

```text
module/package metadata header
        ↓
dependency preamble
        ↓
ordinary module body
```

Once the first ordinary body statement/declaration begins, later `import`, `from … import`, direct `export … from`, or `expose` is a syntax/structural error.

Valid:

```phalcom
import .config
from .models import User
export User from .models

const config = config.load
class App { }
export App
```

Invalid:

```phalcom
const x = 1

import .config
```

Diagnostic:

```text
ImportOutsidePreambleError:
  static imports must appear in the module dependency preamble
```

The parser should own this structural error rather than leaving it to `Compiler::ImportNotAtTopLevel`.

Nested imports remain syntactically invalid.

Comments and blank lines do not terminate the preamble.

Local `export App` remains legal after `App` is declared because it does not introduce a new module dependency.

---

# 16. Import binding scope

Although imports are source-located in the preamble, they are semantically linked before body compilation.

Therefore:

```phalcom
import .point

class C {
    // body may refer to point
}
```

has one module-wide immutable import binding.

The old “enters lexical scope from source position” rule is deleted.

Reordering imports inside the preamble cannot change:

- scope;
- initialization schedule;
- binding identity;
- API;
- runtime behavior.

It may only change diagnostics/source formatting.

---

# 17. Path visibility: private by default

Implement external path visibility in `phalcom-modules/src/interface.rs` and `resolver.rs`.

Each package interface owns:

```rust
pub struct PackagePathSurface {
    pub exposed_children: BTreeSet<ModuleComponent>,
}
```

External resolution algorithm for dependency alias `geo` resolving `geo.shapes.circle`:

1. Resolve alias `geo` to dependency `ResolvedProjectId`.
2. Root package is addressable automatically.
3. Resolve root package interface.
4. Require root package to expose child `shapes`.
5. Resolve package `shapes`.
6. Require `shapes` package to expose child `circle`.
7. Resolve final child.
8. Continue normal module/interface validation.

Same-project imports skip exposure checks.

This yields:

```text
filesystem existence
    ≠
logical path existence to an external project
```

## 17.1 Important façade case

Private module:

```text
geometry/internal/cartesian.ph
```

may export:

```phalcom
export CartesianPoint
```

Public root package may re-export:

```phalcom
export CartesianPoint as Point from .internal.cartesian
```

without:

```phalcom
expose .internal
```

External consumer can use:

```phalcom
from geometry import Point
```

but cannot resolve:

```phalcom
import geometry.internal.cartesian
```

That is the intended encapsulation boundary.

---

# 18. Binding visibility

`ModuleInterface` must record all declarations and a distinct export table.

Do not infer privacy from `_`.

```rust
pub struct ModuleInterface {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub declarations: BTreeMap<String, DeclarationSurface>,
    pub exports: BTreeMap<String, ExportSurface>,
    pub imports: Vec<ImportSurface>,
    pub exposed_children: BTreeSet<ModuleComponent>,
    pub metadata: ModuleMetadata,
}
```

A declared but unexported name is private.

Selective external import:

```phalcom
from geometry.internal_x import Hidden
```

must distinguish:

- target path inaccessible;
- target module accessible but `Hidden` private;
- target module accessible and `Hidden` nonexistent.

Use separate diagnostics.

---

# 19. `from package import X` has one meaning

The linker must never interpret:

```phalcom
from geometry import point
```

as child-module discovery.

Algorithm:

1. resolve `geometry` package;
2. look up exported binding named `point`;
3. succeed or issue export diagnostic.

To import child module:

```phalcom
import geometry.point
```

To make the child module itself a package export:

```phalcom
import .point as point
export point
```

This distinction prevents hidden resolver fallback and keeps completion exact.

---

# 20. Metadata-only module/package attributes

Runtime/transforming module/package decorators are explicitly out of scope.

To avoid ambiguity with existing class/member `@Attribute` syntax, introduce an explicit **file-header target qualifier**:

```phalcom
@module.documentation("Parser implementation")
@module.stability(#experimental)

import .tokens
```

For a package:

```phalcom
@package.documentation("Public geometry façade")
@package.stability(#stable)

export Point from .point
```

These are the only new module/package attribute forms in v1.

The `module` and `package` words after `@` are contextual target qualifiers; they do not need to become globally reserved identifiers.

AST in `phalcom-modules/src/metadata.rs` / `phalcom-ast/src/ast.rs`:

```rust
pub enum MetadataTarget {
    Module,
    Package,
}

pub struct ModuleMetadataAttribute {
    pub target: MetadataTarget,
    pub name: String,
    pub arguments: Vec<MetadataLiteral>,
    pub range: SourceRange,
}
```

`MetadataLiteral` must be inert data:

```rust
pub enum MetadataLiteral {
    Unit,
    Bool(bool),
    Int(String),
    Float(f64),
    String(String),
    Symbol(String),
    Tuple(Vec<MetadataLiteral>),
    Record(Vec<(String, MetadataLiteral)>),
}
```

Do not accept arbitrary `Expr`.

This guarantees that metadata parsing:

- executes no user code;
- performs no message sends;
- imports nothing;
- cannot mutate the module graph;
- is reproducible;
- is safe for LSP/indexing;
- can be cached/serialized.

`@package.*` in an ordinary module is an error. `@module.*` may be allowed for both modules and packages because a package is a specialized module; document whichever convention is chosen consistently. Recommended: allow it, while `@package.*` requires package kind.

Compile-time expanding module/package attributes are a future extension and must use a separate capability/phase mechanism. Do not make this metadata representation secretly executable.

---

# 21. Interface construction

Implement `InterfaceBuilder` in `phalcom-modules/src/interface.rs`.

Input:

```rust
pub struct ParsedModule<'a> {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub program: &'a Program,
}
```

Output:

```rust
pub struct UnlinkedModuleInterface { ... }
```

The builder performs only source-local work:

- collect top-level declarations;
- collect import local names;
- reject duplicate local import bindings;
- collect local export declarations;
- collect direct re-export declarations;
- collect `expose` declarations;
- validate `expose` placement is package-only;
- validate metadata target;
- validate local export refers to a top-level source binding or imported binding;
- record source ranges for diagnostics.

It does **not** resolve target module paths; that is linker/resolver work.

Use source spans in every surface record so the LSP and compiler share diagnostics.

---

# 22. Interface states

Use explicit static states rather than one overloaded “loaded” state.

Recommended pipeline types:

```text
SourceUnit
    ↓ parse
ParsedModule
    ↓ local collection
UnlinkedModuleInterface
    ↓ path resolution
ResolvedModuleInterface
    ↓ symbol/export linking
LinkedModuleInterface
```

Do not encode these as a mutable struct with many `Option` fields if avoidable. Rust types should make phase invariants visible.

This is especially important for future semantic SCC processing: a module can have a known declaration surface before all cross-module symbols are linked.

---

# 23. Errors

Create `phalcom-modules/src/error.rs` with typed `thiserror` enums.

Required project/resolution categories:

```text
InvalidProjectManifestError
InvalidProjectNamespaceError
InvalidDependencyAliasError
ImportRootCollisionError
ProjectDependencyCycleError
UnresolvedPackageDependencyError

ModuleNotFoundError
PackageNotFoundError
InvalidModuleLayoutError
AmbiguousModuleError
InvalidModuleNameError
UnknownImportRootError
RelativeImportBeyondRootError
ImportOutsideSourceRootError
NestedProjectBoundaryError
DuplicateSourceIdentityError

ModulePathNotExposedError
UnknownImportNameError
NonExportedImportError
DuplicateImportBindingError
UnknownExportError
DuplicateExportError
InvalidExposeTargetError
ExposeOutsidePackageError
ImportOutsidePreambleError
InvalidModuleMetadataError
```

Do not collapse access control into “not found” when the compiler knows the target exists.

Example:

```text
ModulePathNotExposedError:
  `geometry.internal.cache` exists, but `internal` is private
  to project `geometry-kit`.

  The dependency exposes:
    geometry
    geometry.point
    geometry.shapes
```

For private binding:

```text
NonExportedImportError:
  `geometry.point` declares `implementationCache`,
  but that binding is not exported.
```

---

# 24. Parser implementation steps

Modify `phalcom-ast/src/token.rs`:

- add `From`;
- add `Export`;
- add `Expose`;
- retain `Import`, `As`, `Dot`, parentheses, comma.

Modify `phalcom-ast/src/lexer.rs::scan_identifier`:

- map exact keyword text to new token variants;
- retain normal identifiers elsewhere.

Modify `phalcom-ast/src/ast.rs`:

- remove old U15 `ImportStatement`;
- add path/import/export/preamble structures from §§12–13;
- add `Statement::Export(ExportDecl)`;
- change `Program` to own `preamble`.

Modify `phalcom-ast/src/parser.rs`:

Add exact helpers:

```rust
fn parse_module_preamble(&mut self) -> ParserResult<ModulePreamble>;
fn parse_import_path(&mut self) -> ParserResult<ImportPath>;
fn parse_module_import(&mut self) -> ParserResult<ImportDecl>;
fn parse_selective_import(&mut self) -> ParserResult<ImportDecl>;
fn parse_import_items(&mut self) -> ParserResult<Vec<ImportItem>>;
fn parse_export_decl(&mut self) -> ParserResult<ExportDecl>;
fn parse_reexport_decl(&mut self) -> ParserResult<ReExportDecl>;
fn parse_export_items(&mut self) -> ParserResult<Vec<ExportItem>>;
fn parse_expose_decl(&mut self) -> ParserResult<ExposeDecl>;
fn parse_module_metadata_header(&mut self) -> ParserResult<Vec<ModuleMetadataAttribute>>;
```

Do not parse an import path as a general expression. It is its own grammar and must remain statically enumerable.

`parse_program` should:

1. consume file-header metadata;
2. consume all dependency-preamble declarations;
3. parse ordinary statements;
4. if a dependency-preamble introducer appears after body start, emit `ImportOutsidePreambleError` and recover.

---

# 25. Resolver implementation steps

In `phalcom-modules/src/resolver.rs` define:

```rust
pub struct ModuleResolver<'u, P: SourceProvider> {
    universe: &'u ProjectUniverse,
    source: &'u P,
    cache: HashMap<ResolutionKey, Result<SourceUnit, ModuleResolutionError>>,
}
```

`ResolutionKey` includes:

```rust
pub struct ResolutionKey {
    pub importer: ModuleId,
    pub path: ImportPathKey,
}
```

Resolution must be pure with respect to runtime state.

Implement:

```rust
pub fn resolve_import(
    &mut self,
    importer: &ModuleId,
    syntax: &ImportPath,
) -> Result<SourceUnit, ModuleResolutionError>;
```

Absolute flow:

```text
syntax root
→ importer project's ImportRootTable
→ target project id
→ project-relative ModulePath
→ SourceProvider::locate
→ external exposure validation if project differs
→ SourceUnit
```

Relative flow:

```text
importer package context
→ apply leading-dot ancestry
→ project-relative ModulePath
→ SourceProvider::locate
```

Relative imports never cross `ResolvedProjectId`.

---

# 26. Public-path validation algorithm

Do not validate only the final module.

Given importer project `A` and target project `B`, path:

```text
geo.shapes.circle
```

validate package boundaries:

```text
B root package
  exposes shapes?
      ↓
B.shapes package
  exposes circle?
      ↓
B.shapes.circle
```

This requires package interfaces to be discoverable before the dependent child path is considered externally accessible.

To avoid a recursive cycle between “resolve interface” and “check visibility”, split physical/source existence from external accessibility:

```text
locate_internal(ModuleId)
    → source unit regardless of cross-project visibility

load_package_surface(ModuleId)
    → only static package metadata/preamble

validate_external_path(...)
    → uses exposed_children surfaces

resolve_external(...)
    → combines both
```

Within a project, use `locate_internal` directly.

This distinction also enables the LSP to diagnose an inaccessible path that really exists.

---

# 27. Project-root and reserved-root policy

Reserve an explicit core root.

Recommended resolver root:

```text
core
```

Bare global lookup may continue to fall back to core according to existing language rules, but explicit access must also be possible through the resolver model.

Dependency alias collision validation must include `core`.

Do not make core a filesystem project. Model it as a reserved provider/project identity so the same `ModuleId` abstraction can represent source-authored core declarations where practical.

The existing LSP constant `CORE_MODULE_URI = "phalcom://core"` should become source/UI metadata, not canonical semantic identity.

---

# 28. LSP identity migration hook

Part II performs the complete LSP graph migration, but Part I must establish the correct identity seam.

Modify `phalcom-lsp/Cargo.toml`:

```toml
phalcom-modules = { path = "../phalcom-modules" }
```

Modify `phalcom-lsp/src/semantic/ids.rs`:

- remove URI-backed `ModuleId(String)`;
- re-export/use `phalcom_modules::identity::ModuleId`;
- retain `ClassId`, `CallableId`, `FieldId`, `DispatchSide`.

Add an LSP-local source mapping:

```rust
pub struct DocumentModuleMap {
    by_uri: BTreeMap<Url, ModuleId>,
    by_module: BTreeMap<ModuleId, Url>,
}
```

The LSP may still key open documents by URI, but semantic identity must no longer equal URI.

All `ModuleId::from_uri` call sites identified in:

- `phalcom-lsp/src/semantic/snapshot.rs`
- `phalcom-lsp/src/semantic/mod.rs`
- `phalcom-lsp/src/semantic/engine.rs`
- `phalcom-lsp/src/semantic/module_graph.rs`
- `phalcom-lsp/src/backend.rs`
- `phalcom-lsp/src/completion.rs`
- `phalcom-lsp/src/inlay_hints.rs`

must migrate through the project/source ownership resolver.

Part II specifies the graph-level replacement.

---

# 29. Source portability validation

Filesystem behavior differs by case sensitivity and Unicode normalization. Semantic identity must not.

During project/package validation detect physical names that collapse under a portability check:

- case-fold collision (`Foo.ph` vs `foo.ph`);
- normalization-equivalent Unicode names;
- `x.ph` vs `x/package.ph`;
- invalid identifier components;
- reserved `package.ph` misuse.

For local compilation, these should at least be diagnostics. Package publication tooling can later make them hard publication errors.

Do not use host filesystem case behavior as language identifier equality.

---

# 30. TDD task sequence

The implementation should proceed in this order.

## Task 1 — identity types

Files:

- create `phalcom-modules/Cargo.toml`
- create `phalcom-modules/src/lib.rs`
- create `phalcom-modules/src/identity.rs`
- modify root `Cargo.toml`

Tests:

```text
ModuleId equality ignores dependency alias spelling.
Distinct ResolvedProjectId values produce distinct ModuleId values.
Root ModulePath is empty and displays through project namespace.
ModuleComponent rejects invalid Phalcom identifiers.
```

Run:

```bash
cargo test -p phalcom-modules
```

## Task 2 — manifest/project universe

Files:

- `phalcom-modules/src/manifest.rs`
- `phalcom-modules/src/project.rs`
- `phalcom-modules/src/error.rs`

Tests:

- default `source = "src"`;
- invalid namespace;
- self/dependency alias collision;
- core alias collision;
- path dependency graph;
- dependency cycle;
- duplicate alias to same resolved project node preserves one identity;
- distinct nodes with same metadata remain distinct.

## Task 3 — source provider

Files:

- `phalcom-modules/src/source.rs`

Tests use temporary directory fixtures:

- explicit `package.ph`;
- missing intermediate `package.ph`;
- `x.ph` / `x/package.ph` ambiguity;
- root package;
- source-root confinement;
- symlink escape where platform supports symlinks;
- nested project boundary;
- standalone package detection.

## Task 4 — AST replacement

Files:

- `phalcom-ast/src/token.rs`
- `phalcom-ast/src/lexer.rs`
- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-ast/tests/lexer.rs`
- `phalcom-ast/tests/parser.rs`
- parser snapshots.

Red tests first for every syntax example in §14.

Delete old physical import parser fixtures; add migration diagnostics only if desired.

## Task 5 — metadata header

Files:

- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-modules/src/metadata.rs`

Tests:

- module metadata header;
- package metadata header;
- arbitrary expression argument rejected;
- metadata after preamble rejected;
- ordinary class attribute remains class attribute.

## Task 6 — interface builder

Files:

- `phalcom-modules/src/interface.rs`

Tests:

- private-by-default declarations;
- explicit exports;
- export alias;
- direct re-export;
- duplicate export;
- duplicate import binding;
- expose allowed only for package;
- expose only immediate child.

## Task 7 — resolver/path privacy

Files:

- `phalcom-modules/src/resolver.rs`

Tests:

- absolute self import;
- dependency alias import;
- relative sibling;
- relative parent;
- beyond root;
- cross-project private child rejection;
- hierarchical exposure;
- private implementation module re-export through public façade;
- `from package import X` never child-fallbacks.

---

# 31. Performance requirements

Part I is compile/tooling infrastructure, but its design directly controls startup cost.

Required properties:

1. No runtime work is performed for resolution.
2. No source import causes filesystem search-path enumeration.
3. Resolved `(importer, path)` pairs are memoized.
4. Project manifests are parsed once per project universe generation.
5. Source roots are canonicalized once.
6. Package-marker/directory probes are cached.
7. `ModuleComponent` validation happens at construction, not at every comparison.
8. `ModuleId` equality/hash does not canonicalize paths.
9. LSP uses the same resolver semantics rather than maintaining a second path algorithm.
10. The compiler traverses only statically reachable source, not every `.ph` file in a project.
11. Public-path checks use already-built package surfaces and in-memory sets.
12. No attribute metadata requires VM startup or user-code execution.

Benchmark/measure cold project resolution separately from runtime benchmarks. A module-system regression must not be hidden by VM execution cost.

---

# 32. Invariants to assert in code

Use `debug_assert!`/typed construction where invariants are programmer errors:

```text
I1. Every ModuleId references exactly one ResolvedProjectId.
I2. Every project-backed ModulePath is relative to exactly one source root.
I3. Root ModulePath resolves only to source-root/package.ph.
I4. Every package ModuleId is backed by package.ph.
I5. An ordinary module and package never share one ModuleId.
I6. Relative import resolution never changes ResolvedProjectId.
I7. Cross-project deep resolution crosses only exposed child boundaries.
I8. Same-project resolution ignores public exposure.
I9. Binding export does not imply path exposure.
I10. Path exposure does not imply binding export.
I11. Dependency aliases never enter ModuleId.
I12. SourceId never substitutes for ModuleId.
I13. Dependency-preamble order is semantically irrelevant.
I14. Module/package metadata is inert data only.
```

---

# 33. Documentation changes

When implementation lands, revise:

- `docs/spec/next/modules-next.md` into the new normative static model;
- `docs/spec/lexical-structure.md` for `from`, `export`, `expose`, dotted import paths, grouped forms;
- module/package/project learning material;
- migration notes for U15 physical imports.

Write a PDR/ADR capturing at least:

```text
- semantic ModuleId != SourceId
- path privacy separate from binding privacy
- project-private child paths by default
- static dependency preamble
- no ambient resolution
- package containment != initialization dependency
```

The rationale matters because these are compatibility-sensitive boundaries.

---

# 34. Acceptance criteria for Part I

Part I is complete when all of the following are true:

- `phalcom-modules` exists and is VM-free.
- `ModuleId` is project-instance + project-relative path.
- filesystem paths/URIs are no longer semantic module identity in new code.
- `project.toml` parses and validates project namespace/source/entry/dependencies.
- explicit packages and module/package collisions are validated.
- logical absolute and relative import paths parse.
- physical quoted import syntax no longer parses as an import.
- grouped selective imports parse.
- grouped local exports parse.
- direct/grouped re-exports parse.
- `expose .child` parses and is package-only.
- imports/re-exports/exposes form a dependency preamble.
- module/package metadata headers are inert and literal-only.
- module interfaces contain declarations, exports, imports, exposure surface, and metadata.
- cross-project child paths are private by default.
- hierarchical exposure is enforced.
- package façade re-export from a private implementation path works semantically.
- LSP has a semantic identity seam that no longer requires URI == ModuleId.
- no runtime import opcode/path loader has been added to make the new syntax work.

Part II may begin only after these static invariants are testable without constructing a `VM`.

---

# 35. Explicitly deferred from Part I

Do not implement here:

- runtime module initialization;
- runtime lazy/dynamic loading;
- reload;
- namespace packages;
- wildcard imports/exports;
- environment search paths;
- module-path mutation;
- runtime module/package decorators;
- compile-time expanding module/package decorators;
- package manager registry acquisition/version solving;
- workspaces;
- multiple executable targets;
- compiled-module file syntax.

The interfaces above deliberately leave those features addable without changing canonical module identity or source import semantics.
