# Telos Package Model and `Archon.toml`

This specification defines the initial Telos package model, the target kinds owned by a package, and the manifest language used by Archon.

It is intentionally centered on the semantic model rather than on filesystem convention.

The foundational rule is:

> **A package is the persistent semantic compilation and compiler-artifact ownership boundary. Targets are named products or development overlays derived from that package; they are not independent compiler projects.**

The initial target set is fixed as:

```text
lib      0..1
bin      0..N
test     0..N
example  0..N
bench    0..N
```

---

## Package identity

Every persistent Telos package is declared by exactly one `Archon.toml`.

A minimal package declaration is:

```toml
[package]
name = "parse"
namespace = "parse"
version = "1.2.0"
```

The three fields have distinct roles.

### `name`

The package's human/package-manager identity.

Example:

```toml
name = "parse"
```

Package names participate in registry publication and package dependency declarations.

### `namespace`

The canonical root namespace of the package's Telos module world.

Example:

```toml
namespace = "parse"
```

A package named:

```text
parse-tools
```

may therefore choose a module namespace such as:

```text
parse_tools
```

Package name and namespace MUST NOT be treated as interchangeable strings merely because many packages choose equivalent names.

### `version`

The package release version.

Example:

```toml
version = "1.2.0"
```

The initial package ecosystem uses semantic-version-style package versions.

A registry release binds this version to immutable release authority.

A mutable path dependency may contain the same version metadata, but its active dependency authority remains the path checkout rather than a published release.

---

## A package is one compiler world

A package owns one production module/semantic world.

For example:

```text
atlas package
├── atlas
├── atlas.http
├── atlas.config
├── atlas.internal.storage
├── atlas.cli
└── atlas.server
```

may expose:

```text
lib atlas
bin atlas        -> atlas.cli
bin atlas-server -> atlas.server
```

but this does not imply three independent compiler units.

Conceptually:

```text
atlas source
    │
    ▼
one package ModuleWorld
    │
    ▼
one production semantic generation
    │
    ├── library exposure
    ├── executable closure for `atlas`
    └── executable closure for `atlas-server`
```

The package compiler artifact is correspondingly package-scoped:

```text
atlas.telosc
```

rather than:

```text
atlas-lib.telosc
atlas-cli.telosc
atlas-server.telosc
```

Target-specific executable products may exist separately, but they do not redefine compiler-artifact ownership.

---

# Library target

A package may declare zero or one library target.

```toml
[lib]
root = "parse"
```

`root` identifies the canonical module root through which the package is consumed as a library.

The library target is not a second compiler project.

It defines the package's externally consumable module surface.

## External visibility

A dependent package may only consume declarations and child modules that are externally exposed according to the Telos package/module visibility rules.

For example:

```text
parse
├── Rule                    public
├── Parser                  public
├── combinators             exposed
│   ├── choice
│   └── sequence
└── internal                not exposed
    └── scanner
```

A dependent package may be able to use:

```telos
import parse.Rule
import parse.combinators.choice
```

while being forbidden from importing:

```telos
import parse.internal.scanner
```

even though the latter may physically exist in the package artifact.

Artifact presence MUST NOT imply public accessibility.

## Cardinality

The initial package model MUST permit at most one `[lib]`.

A package requiring two independently versioned/importable library surfaces SHOULD be represented as two packages unless future language/package experience establishes a stronger use case for multiple library targets.

---

# Binary targets

A package may declare zero or more binary targets.

```toml
[[bin]]
name = "atlas"
entry = "atlas.cli"

[[bin]]
name = "atlas-server"
entry = "atlas.server"
```

Each binary has:

- a target name used by Archon;
- one canonical Telos entry module;
- a target-specific executable/runtime dependency closure.

## Entry is a module identity

`entry` MUST identify a canonical Telos module.

It MUST NOT identify:

- a source filename;
- an arbitrary source directory;
- a magic `main()` function;
- a host-language function symbol.

For:

```toml
entry = "atlas.server"
```

the compiler executes a program rooted at module:

```text
atlas.server
```

The package/module system determines initialization and runtime dependencies.

## No mandatory `main()` callable

The initial package model does not introduce a special language-level `main()` function contract.

An entry module may, for example, contain ordinary top-level program behavior:

```telos
let config = Config.load()
Server(config).run()
```

A package author may choose to structure the module around an ordinary function named `main`, but that is style, not compiler entry semantics.

This preserves a clean rule:

> **Executable entry is a module-system concept, not a special callable-system exception.**

## Same-package authority

Unlike tests, examples, and benchmarks, a binary target is an ordinary production target of its package.

Its module participates in normal same-package visibility according to the Telos visibility rules.

The binary is not restricted to consuming only the library's external surface.

---

# Choosing the default binary

A package with a binary may optionally declare:

```toml
[package]
name = "atlas"
namespace = "atlas"
version = "1.0.0"
default-run = "atlas"
```

`default-run` names a declared `[[bin]]` target.

Archon MUST resolve `archon run` as follows:

1. If `package.default-run` exists, select that binary.
2. Otherwise, if exactly one binary exists, select it.
3. Otherwise, fail and require explicit target selection.

For a package with:

```toml
[[bin]]
name = "client"
entry = "net.client"

[[bin]]
name = "server"
entry = "net.server"
```

and no `default-run`, this is invalid:

```bash
archon run
```

unless Archon produces an ambiguity error describing the available targets.

The explicit form is:

```bash
archon run --bin server
```

Changing only `default-run` is Archon target-selection policy and SHOULD NOT by itself invalidate the package's semantic compiler artifact.

---

# Integration test targets

The initial testing system is deliberately narrow.

A package may declare zero or more test targets:

```toml
[[test]]
name = "parser"
entry = "parse_tests.parser"

[[test]]
name = "unicode"
entry = "parse_tests.unicode"
```

The first testing model has the following hard constraints:

- tests are integration tests only;
- test execution is always managed by an Archon test harness;
- there are no unit-test declarations embedded in production modules;
- there are no documentation tests;
- there is no harness-free test target mode;
- tests receive no privileged package-internal visibility;
- tests may use normal dependencies and development dependencies;
- tests are development overlays, not part of the production package semantic image.

## Integration-test visibility

A test target consumes the package through its library surface.

Conceptually:

```text
production package generation
        │
        ▼
published/exposed library surface
        │
        ▼
test overlay
```

It does not become a privileged same-package source unit merely because it lives in the same repository.

If:

```text
parse.internal.Scanner
```

is not externally available through the library surface, an integration test MUST NOT gain access to it through a test-only visibility bypass.

This rule deliberately avoids introducing a second visibility model for tests.

## Unit-sized testing through integration

The absence of a unit-test feature does not prevent narrow tests.

A test may exercise one small exported abstraction:

```text
tests/scanner
```

or one narrow behavior of a public package API.

The distinction is architectural:

> The compiler does not need a separate "test code inside production module" semantic mode.

Unit-test syntax, private-access policy, source annotations, and documentation tests are deferred.

## Archon harness requirement

Every initial test target is harness-managed.

The conceptual execution flow is:

```text
test target module
     │
     ▼
Telos compilation
     │
     ▼
test-case discovery / registration contract
     │
     ▼
Archon harness
     │
     ├── scheduling
     ├── execution
     ├── result aggregation
     └── reporting
```

The exact test-case declaration API is outside this manifest specification, but the target model assumes a structured Archon-managed harness.

The manifest MUST NOT initially support:

```toml
harness = false
```

---

# Example targets

A package may declare zero or more examples.

```toml
[[example]]
name = "basic"
entry = "parse_examples.basic"

[[example]]
name = "json"
entry = "parse_examples.json"
```

An example is an executable development target.

It resembles a binary in that it has an entry module and can be run.

It differs from a normal package binary in authority:

> An example is treated as a development consumer of the package's library surface.

Conceptually:

```text
package library generation
        │
        ▼
example overlay
```

An example MUST NOT gain privileged access to hidden package internals merely because it is maintained in the same source repository.

This gives examples two useful properties:

1. they demonstrate realistic consumer usage;
2. they act as executable checks of the public API.

Examples may use development dependencies.

Typical commands are expected to be:

```bash
archon run --example basic
archon build --example basic
archon build --examples
```

The command syntax is specified by the Archon CLI specification; this document specifies only target semantics.

---

# Benchmark targets

A package may declare zero or more benchmark targets.

```toml
[[bench]]
name = "throughput"
entry = "parse_benches.throughput"
```

A benchmark is a development consumer of the package library surface, just like an example or integration test.

It may use development dependencies.

Benchmark target compilation is owned by Telos, while benchmark orchestration is owned by Archon.

Archon is responsible for concerns such as:

- warm-up;
- repetition count;
- timing strategy;
- sample collection;
- statistical aggregation;
- baseline comparison;
- user-facing reporting.

These are tooling semantics, not Telos language semantics.

A benchmark target MUST NOT become part of the production package dependency surface.

---

# Production targets and development overlays

The initial target kinds fall into two groups.

## Production package targets

```text
lib
bin
```

These belong to the normal package compilation world.

## Development overlays

```text
test
example
bench
```

These are compiled against the normal package authority without redefining it.

This distinction has several consequences.

### Production dependency graph

Normal package compilation uses:

```text
[dependencies]
```

### Development target dependency graph

Tests, examples, and benchmarks use:

```text
[dependencies]
+
[dev-dependencies]
```

### Package artifact purity

Running:

```bash
archon test
```

or:

```bash
archon bench
```

MUST NOT cause test/benchmark declarations to become part of the canonical published package compiler artifact.

Conceptually:

```text
PackageGeneration P
    │
    ├── freeze -> P.telosc
    │
    ├── + TestOverlay T -> PT
    ├── + ExampleOverlay E -> PE
    └── + BenchOverlay B -> PB
```

`PT`, `PE`, and `PB` are development build generations.

They MUST NOT be mistaken for the canonical package generation `P`.

---

# Dependencies

`Archon.toml` supports two fundamentally different dependency authorities.

## Registry dependency

The short form:

```toml
[dependencies]
parse = "1.2"
```

means:

> Resolve package `parse` from the configured package registry using version requirement `1.2`.

The explicit form is:

```toml
[dependencies]
parse = { version = "^1.2" }
```

A dependency alias may differ from the registry package name:

```toml
[dependencies]
parser = { package = "parse", version = "^1.2" }
```

Then Telos source refers to the dependency through the resolved dependency alias according to module/import rules.

Registry resolution semantics, candidate selection, lockfile authority, and publication are owned by the registry specifications.

## Path dependency

A mutable development dependency uses:

```toml
[dependencies]
parse = { path = "../parse" }
```

This means:

> Resolve the package rooted at that local path as mutable development authority.

A path dependency is not a locally published registry release.

Archon MUST NOT silently assign registry-release immutability semantics to a path checkout.

## Path and registry authority are mutually exclusive

A dependency declaration MUST NOT combine:

```toml
path = "../parse"
```

with:

```toml
version = "1.2"
```

or:

```toml
package = "parse"
```

as competing acquisition authorities.

A dependency entry is either:

```text
path authority
```

or:

```text
registry authority
```

not both.

---

# Development dependencies

Development-only dependencies are declared separately:

```toml
[dev-dependencies]
test-support = "0.3"
property = "^1.0"
```

They are available to:

```text
test
example
bench
```

They MUST NOT automatically become dependencies of:

```text
lib
bin
```

This separation prevents a package from acquiring production dependency edges merely because its validation/examples/benchmarks require additional tooling.

Path dependencies are also permitted here:

```toml
[dev-dependencies]
test-support = { path = "../test-support" }
```

---

# Target dependency and visibility matrix

The initial rules are:

| Target | Package library | Normal dependencies | Development dependencies | Special access to package internals |
|---|---:|---:|---:|---:|
| `lib` | self | Yes | No | ordinary same-package rules |
| `bin` | same package | Yes | No | ordinary same-package rules |
| `test` | library consumer | Yes | Yes | No |
| `example` | library consumer | Yes | Yes | No |
| `bench` | library consumer | Yes | Yes | No |

This distinction is normative.

---

# Manifest source structure

A package may use conventional directories such as:

```text
project/
├── Archon.toml
├── Archon.lock
├── src/
├── tests/
├── examples/
└── benches/
```

The toolchain MAY provide directory-based target defaults later.

However:

> Physical file placement is not canonical package/module identity.

`entry` fields use canonical Telos module identities.

The source-provider/module system remains responsible for mapping package modules to source units.

This specification deliberately avoids defining:

```text
tests/foo.ph => target identity "foo"
```

as an unconditional semantic rule.

Such conveniences, if added, are Archon discovery conventions that must normalize into the same target model defined here.

---

# Initial `Archon.toml` form

A representative complete manifest is:

```toml
[package]
name = "parse"
namespace = "parse"
version = "1.2.0"
default-run = "parse"

[lib]
root = "parse"

[[bin]]
name = "parse"
entry = "parse.cli"

[[bin]]
name = "parse-server"
entry = "parse.server"

[[test]]
name = "parser"
entry = "parse_tests.parser"

[[test]]
name = "combinators"
entry = "parse_tests.combinators"

[[example]]
name = "basic"
entry = "parse_examples.basic"

[[example]]
name = "json"
entry = "parse_examples.json"

[[bench]]
name = "throughput"
entry = "parse_benches.throughput"

[dependencies]
text = "^1.0"
http = { version = "^2.0" }

[dev-dependencies]
test-support = "0.3"
property = "^1.0"
```

This describes one package, not eight independent compilation projects.

---

# Library-only package

A library package may be:

```toml
[package]
name = "slotmap"
namespace = "slotmap"
version = "0.1.1"

[lib]
root = "slotmap"
```

It may still have tests, examples, and benchmarks:

```toml
[[test]]
name = "runtime"
entry = "slotmap_tests.runtime"

[[example]]
name = "basic"
entry = "slotmap_examples.basic"

[[bench]]
name = "lookup"
entry = "slotmap_benches.lookup"
```

Since no binary exists:

```bash
archon run
```

must fail unless an example is explicitly selected with the appropriate Archon option.

---

# Binary-only package

A binary-only package is valid:

```toml
[package]
name = "db-migrate"
namespace = "db_migrate"
version = "0.1.0"

[[bin]]
name = "db-migrate"
entry = "db_migrate.main"
```

Such a package has no library surface for ordinary package dependency consumption.

Its internal module graph may still be large:

```text
db_migrate
db_migrate.schema
db_migrate.database
db_migrate.main
```

but external packages cannot depend on a nonexistent `[lib]` surface.

---

# Mixed library and binary package

A common package shape is:

```toml
[package]
name = "rivet"
namespace = "rivet"
version = "2.0.0"
default-run = "rivet"

[lib]
root = "rivet"

[[bin]]
name = "rivet"
entry = "rivet.cli"

[[bin]]
name = "rivet-language-server"
entry = "rivet.lsp"
```

Both binaries share the package's semantic world.

Target-specific runtime lowering happens after package semantic analysis.

The implementation SHOULD maximize sharing of target-independent semantic products rather than compiling each binary as an isolated package.

---

# Package target validity

A normal package MUST initially define at least one production target:

```text
lib
or
bin
```

A manifest containing only:

```text
test
example
bench
```

is invalid in the initial model.

Test-only fixtures may be reconsidered later if real use cases justify them.

---

# Name uniqueness

Target names MUST be unique within their target kind.

For example, this is invalid:

```toml
[[bin]]
name = "server"
entry = "foo.server"

[[bin]]
name = "server"
entry = "foo.server_debug"
```

Archon SHOULD also reject or strongly constrain cross-kind ambiguities when they would make CLI selection unclear.

For example:

```text
bin/server
example/server
```

can technically be disambiguated through `--bin` versus `--example`, but implementations should preserve stable fully qualified target identity:

```text
TargetId(package, kind, name)
```

rather than assuming target name alone is globally unique.

---

# Entry validity

Each entry-bearing target MUST reference a canonical module identity appropriate to its package or overlay.

## Binary

A binary entry belongs to the package's production module world.

Example:

```toml
entry = "atlas.server"
```

## Test

A test entry belongs to the test/development overlay.

Example:

```toml
entry = "atlas_tests.protocol"
```

## Example

An example entry belongs to the example/development overlay.

```toml
entry = "atlas_examples.basic"
```

## Benchmark

A benchmark entry belongs to the benchmark/development overlay.

```toml
entry = "atlas_benches.throughput"
```

The exact canonical overlay namespace convention may be finalized by the source/module-provider specification. The essential invariant here is that `entry` is resolved semantically as a module identity, not interpreted as an arbitrary file path.

---

# `default-run` validity

If present:

```toml
default-run = "server"
```

MUST name exactly one declared binary target.

It MUST NOT select:

```text
test
example
bench
lib
```

The normal `archon run` command means "run the package's selected binary."

Examples use explicit example selection.

Tests and benchmarks use their dedicated command families.

---

# Version requirement syntax

Registry dependencies are version requirements, not necessarily exact versions.

The package model is intended to support semantic-version-style requirements such as:

```toml
parse = "1.2"
parse = "^1.2"
parse = "~1.2.3"
parse = "=1.2.4"
```

The registry specification owns exact matching/selection semantics.

`Archon.toml` MUST preserve the distinction between:

```text
requested version requirement
```

and:

```text
exact release chosen in Archon.lock
```

---

# Manifest validation failures

An implementation MUST reject malformed package structure rather than silently normalizing conflicting declarations.

At minimum, validation must detect:

- missing `[package]`;
- invalid package name;
- invalid namespace;
- invalid package version;
- neither `[lib]` nor any `[[bin]]`;
- more than one library declaration;
- duplicate target names within a target kind;
- missing target name;
- missing target entry where required;
- invalid entry module identity;
- `default-run` naming no binary;
- dependency declaring both `path` and registry authority;
- registry dependency without a version requirement;
- duplicate dependency aliases;
- invalid dependency alias/module-root mapping;
- conflicting normal and development dependency declarations where the implementation cannot define an unambiguous merge.

Errors SHOULD identify the relevant manifest field and explain the violated semantic rule.

---

# Production build identity and target metadata

The manifest contains both semantic inputs and orchestration-only settings.

Implementations MUST NOT assume every textual manifest change invalidates every compiler product.

For example:

```toml
default-run = "server"
```

is target-selection policy.

Changing it to:

```toml
default-run = "client"
```

does not by itself change either binary's source semantics.

Conversely, changing:

```toml
entry = "foo.server"
```

to:

```toml
entry = "foo.server_v2"
```

changes the selected binary target definition.

The build/cache specification will define the exact fingerprint partitions, but this package specification requires consumers to distinguish semantic package configuration from Archon UX/orchestration configuration.

---

# Published package surface

A registry-published package release contains the package's production source authority and manifest.

Its canonical package compiler artifact, when available, represents the production package generation.

Development targets do not become dependency surface merely because their declarations exist in the source release.

Downstream dependency consumption is through `[lib]`.

A package with no `[lib]` is therefore not a normal importable library dependency in the initial package model.

---

# Deferred package features

The initial model intentionally excludes:

- multiple library targets;
- unit-test declarations inside `src`;
- documentation tests;
- harness-free tests;
- test-only visibility bypasses;
- target-specific independent package versions;
- importing another package's binary target as a language dependency;
- benchmark-specific dependency sections;
- example-specific dependency sections;
- independently published test/example/benchmark artifacts.

These are deferred rather than implicitly supported.

---

# Migration from the current `project.toml` model

The current compiler repository contains a simpler project manifest shape with fields such as:

```toml
[project]
name = "slotmap"
namespace = "slotmap"
version = "0.1.1"
entry = "slotmap.main"
```

The intended Archon model separates package identity from target declaration.

An executable package becomes:

```toml
[package]
name = "slotmap"
namespace = "slotmap"
version = "0.1.1"

[[bin]]
name = "slotmap"
entry = "slotmap.main"
```

If that `entry` existed only to run package validation/runtime tests rather than to define an actual user executable, it should instead migrate toward an integration test target:

```toml
[package]
name = "slotmap"
namespace = "slotmap"
version = "0.1.1"

[lib]
root = "slotmap"

[[test]]
name = "runtime"
entry = "slotmap_tests.runtime"
```

The older distinction between:

```text
entry
default_entry
```

should not survive as the package-target model.

The new concepts are:

```text
target.entry
    semantic root of one target

package.default-run
    Archon policy for selecting among binary targets
```

These are different responsibilities and should remain separate.

---

# Canonical package model

The complete initial model is therefore:

```text
PackageIdentity
    name
    namespace
    version

ProductionPackageWorld
    normal dependencies
    module world
    semantic generation

Targets
    lib      0..1
    bin      0..N
    test     0..N
    example  0..N
    bench    0..N

DevelopmentEnvironment
    normal dependencies
    + dev dependencies

Publication
    production source authority
    + package manifest
    + optional compatible derived .telosc

Artifact boundary
    one package
    one completed production compiler generation
    one canonical package .telosc
```

The package architecture is intentionally familiar at the Archon surface while differing from Cargo at the compiler boundary:

> **`lib`, `bin`, `test`, `example`, and `bench` are target kinds of one package semantic world—not independent compiler crates.**
