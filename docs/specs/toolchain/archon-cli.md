# `archon` Command-Line Interface

This specification defines the initial command-line interface of Archon, the Telos project, package, build, dependency, registry, test, benchmark, cache, and toolchain orchestrator.

The governing rule is:

> **Archon owns reproducible project realization.**

Archon answers:

```text
What package is being operated on?
Which dependency releases are selected?
Which toolchain and Universe are active?
Which target should be built or executed?
Which cached products can be reused?
Which source must be compiled?
Which artifacts should be persisted?
```

It then constructs a resolved compiler request and invokes Telos.

---

# 1. The initial command surface

The planned initial command families are:

```text
archon new
archon init

archon check
archon build
archon run
archon test
archon bench
archon repl

archon add
archon remove
archon update
archon fetch
archon tree

archon publish

archon artifact ...
archon cache ...
archon registry ...
archon package ...
archon toolchain ...

archon clean

archon fmt
archon doc
```

Not every secondary option needs to ship in the first implementation milestone, but the command ownership and namespace should be preserved.

---

# 2. Project discovery

When run inside a package, Archon discovers the nearest applicable:

```text
Archon.toml
```

according to workspace/project discovery rules.

Archon—not Telos—owns this discovery.

An explicit manifest/path option should be supported for automation and disambiguation.

Conceptually:

```bash
archon --manifest-path /work/foo/Archon.toml build
```

or an equivalent global option.

Archon must not accidentally select an unrelated ancestor package when an explicit manifest has been supplied.

---

# 3. `archon new`

Creates a new Telos package skeleton.

Examples:

```bash
archon new hello
archon new hello --lib
archon new hello --bin
```

The default package shape should be simple and explicit.

A binary default may create:

```text
hello/
├── Archon.toml
└── src/
    └── ...
```

with:

```toml
[package]
name = "hello"
namespace = "hello"
version = "0.1.0"

[[bin]]
name = "hello"
entry = "hello.main"
```

A library package may create:

```toml
[package]
name = "hello"
namespace = "hello"
version = "0.1.0"

[lib]
root = "hello"
```

The exact source file convention is controlled by the package/module model.

---

# 4. `archon init`

Initializes an existing directory as an Archon package.

It creates:

```text
Archon.toml
```

and any required initial source structure.

It MUST NOT overwrite an existing valid manifest without an explicit destructive option.

It should detect whether the directory already contains plausible Telos source and avoid inventing conflicting module identities.

---

# 5. `archon check`

`archon check` performs a semantic build without requiring final executable output.

Conceptually:

```text
discover package/workspace
        ↓
read Archon.toml
        ↓
read/create/validate Archon.lock as policy allows
        ↓
resolve toolchain
        ↓
resolve Universe
        ↓
resolve dependency graph
        ↓
consult artifacts/caches
        ↓
construct ResolvedBuildPlan
        ↓
invoke Telos semantic compilation
        ↓
persist project-local incremental state
        ↓
report diagnostics
```

---

## 5.1 Default check scope

Initial default:

> `archon check` checks the package's production world.

That includes:

```text
lib
bin targets
normal production modules
```

It does not automatically require compilation of:

```text
test
example
bench
```

because those are development overlays with dev dependencies.

A future or initial explicit option may provide:

```bash
archon check --all-targets
```

or target-specific checks.

This preserves fast ordinary feedback while still permitting full validation.

---

## 5.2 Target-specific check

Archon may support:

```bash
archon check --lib
archon check --bin server
archon check --test parser
archon check --example basic
archon check --bench throughput
```

When a development target is selected, dev dependencies are included.

The base package production generation should still be reused.

---

## 5.3 Check does not require artifact freeze

A successful `archon check` may persist project-local incremental compiler state.

It does not inherently require:

```text
new canonical package.telosc
```

This distinction is important for fast edit/check cycles.

---

# 6. `archon build`

`archon build` produces durable build products.

At minimum it:

```text
resolves exact inputs
compiles the required package generation
freezes package artifact when build policy requires it
emits requested target products
updates build indexes only after success
```

---

## 6.1 Default production build

For a package with:

```text
lib
bin A
bin B
```

the default production build should validate/build the production package and configured default production outputs.

The implementation should make this behavior explicit and stable.

A reasonable initial rule is:

```text
archon build
    builds the production package artifact
    and all declared binary targets
```

This provides a predictable "build the package" behavior.

Target-specific forms avoid unnecessary work.

---

## 6.2 Target selection

The initial target selection grammar is:

```bash
archon build --lib

archon build --bin server
archon build --bins

archon build --test parser
archon build --tests

archon build --example basic
archon build --examples

archon build --bench throughput
archon build --benches
```

Plural selectors mean all targets of that kind.

Conflicting selectors should be diagnosed unless the CLI explicitly supports multiple target groups in one invocation.

---

## 6.3 Production versus development build

These selectors:

```text
--lib
--bin
--bins
```

operate on the production package world.

These selectors:

```text
--test
--tests
--example
--examples
--bench
--benches
```

construct development overlays and include dev dependencies.

Development overlay compilation MUST NOT replace the canonical production package artifact with an overlay artifact.

---

# 7. `archon run`

`archon run` builds if required and executes one binary target.

Selection rules are:

1. if `package.default-run` names a binary, select it;
2. otherwise if exactly one binary exists, select it;
3. otherwise fail and require `--bin`.

Examples:

```bash
archon run
archon run --bin server
```

Program arguments follow `--`:

```bash
archon run --bin server -- --port 8080
```

Archon consumes arguments before `--`.

The program receives those after it.

---

## 7.1 Example execution

Examples are runnable but are explicitly development targets.

Use:

```bash
archon run --example basic
```

Do not make `archon run` silently select an example when there is no binary.

A package with only a library and examples should produce a clear error for plain:

```bash
archon run
```

and suggest explicit example selection.

---

# 8. `archon test`

The initial testing model is intentionally narrow:

> **Archon runs integration tests through an Archon-managed harness.**

No initial support exists for:

```text
unit tests embedded in production modules
documentation tests
harness-free test targets
test-only private visibility
```

---

## 8.1 Running all tests

```bash
archon test
```

means:

```text
resolve production dependencies
resolve dev dependencies
build/import base package generation
build all declared integration-test overlays
discover/register test cases through the harness contract
execute
aggregate
report
```

---

## 8.2 Selecting a test target

```bash
archon test --test parser
```

runs only the named integration-test target.

Compilation-only is available through:

```bash
archon build --test parser
```

or:

```bash
archon build --tests
```

---

## 8.3 Test-case filtering

A positional filter may be supported:

```bash
archon test parse_integer
```

meaning:

> Run test cases whose fully qualified names match the filter.

An explicit equivalent may exist:

```bash
archon test --filter parse_integer
```

The precise matching rules should be deterministic and documented.

Target selection remains separate:

```bash
archon test --test parser parse_integer
```

selects target `parser` and filters test cases within the selected test set.

---

## 8.4 Test process output

Archon owns:

```text
test enumeration
scheduling
pass/fail/skip aggregation
summary formatting
filtering
parallelism policy
```

Telos owns language/runtime execution.

The test harness should consume structured results rather than scraping arbitrary stdout where practical.

Program/test stdout and harness status output should remain distinguishable.

---

# 9. `archon bench`

`archon bench` runs Archon-managed benchmark targets.

Examples:

```bash
archon bench
archon bench --bench throughput
```

Compilation-only:

```bash
archon build --bench throughput
archon build --benches
```

Archon owns benchmark mechanics such as:

```text
warm-up
iterations
measurement window
sample collection
summary statistics
baseline comparison
report formatting
```

Telos owns compilation/execution.

---

## 9.1 Benchmark filters

As benchmarks gain multiple cases, Archon may support:

```bash
archon bench parser
```

or:

```bash
archon bench --filter parser
```

with semantics parallel to test filtering.

The first implementation need not include advanced statistical options before the harness is stable.

---

# 10. `archon repl`

`archon repl` starts a Telos REPL in the resolved current package environment.

That means Archon resolves:

```text
toolchain
Universe
production dependencies
package import roots
current package context
```

and then starts Telos's interactive compiler/runtime session.

The REPL must not independently resolve a different dependency graph.

---

# 11. `archon add`

Adds a dependency to `Archon.toml` and resolves the graph.

Registry dependency:

```bash
archon add parse@1.2
```

should produce a manifest entry equivalent to:

```toml
[dependencies]
parse = "1.2"
```

or a normalized explicit form.

Path dependency:

```bash
archon add parse --path ../parse
```

produces:

```toml
[dependencies]
parse = { path = "../parse" }
```

Development dependency:

```bash
archon add test-support@0.3 --dev
```

produces:

```toml
[dev-dependencies]
test-support = "0.3"
```

---

## 11.1 Alias support

A future/initial option should support aliasing:

```bash
archon add parser --package parse --version 1.2
```

or a more ergonomic equivalent.

The resulting manifest might be:

```toml
parser = { package = "parse", version = "1.2" }
```

CLI syntax should avoid ambiguity between alias and package identity.

---

## 11.2 Transactional manifest updates

`archon add` must not leave:

```text
Archon.toml modified
Archon.lock inconsistent
```

after a resolution failure without clearly defined recovery semantics.

Preferred behavior:

```text
prepare manifest update
attempt resolution
write manifest + lock transactionally
```

or preserve enough backup/state to guarantee consistency.

The exact filesystem transaction mechanism is implementation detail.

---

# 12. `archon remove`

Removes a dependency declaration.

Example:

```bash
archon remove parse
```

By default it removes from the dependency section where the alias exists.

If the same alias can exist in multiple dependency classes, require explicit disambiguation such as:

```bash
archon remove test-support --dev
```

After mutation, Archon updates the lock graph accordingly.

It must not delete the package from the registry.

---

# 13. `archon update`

`archon update` changes exact locked registry selections while respecting manifest requirements.

Examples:

```bash
archon update
archon update parse
```

Default:

```text
re-solve eligible registry dependencies
update Archon.lock
do not edit manifest requirements
```

Updating one package may necessarily update transitive dependencies to produce a valid graph.

Archon should report the actual changes.

---

# 14. `archon fetch`

`archon fetch` ensures that the package graph's required immutable release/source material is locally available.

It does not build the root package.

Conceptually:

```text
resolve lock graph
materialize/download required registry release data
populate acquisition cache as appropriate
```

For the first local registry, this may mostly verify/materialize local release objects.

The command remains useful because its semantics extend naturally to future remote registries.

---

# 15. `archon tree`

Displays the resolved dependency graph.

Example:

```bash
archon tree
```

Conceptual output:

```text
atlas 1.0.0
├── parse 1.4.1 (local)
│   └── text 1.2.0 (local)
└── http 2.3.0 (local)
    └── text 1.2.0 (local)
```

Path dependencies must be distinguishable:

```text
util 0.2.0 (path ../util)
```

Development dependencies should be suppressible/includable explicitly.

Potential options:

```bash
archon tree --dev
archon tree --all
```

---

# 16. `archon publish`

Publishes the current package as an immutable registry release.

Default registry initially:

```text
local
```

Thus:

```bash
archon publish
```

is equivalent to:

```bash
archon publish --registry local
```

until additional registries exist.

---

## 16.1 Publication workflow

Archon owns the full orchestration:

```text
validate package manifest
        ↓
validate package target model
        ↓
resolve production dependency requirements
        ↓
perform required package checks/build validation
        ↓
construct canonical source snapshot
        ↓
compute source digest
        ↓
optionally build/validate compatible .telosc
        ↓
construct PublishCandidate
        ↓
invoke registry publish transaction
```

The registry owns immutable storage/conflict handling.

Telos owns compiler validation/building.

---

## 16.2 Re-publishing an existing version

Publishing different source under an existing:

```text
registry/package/version
```

MUST fail.

Archon must not expose a normal:

```text
--force
```

that mutates immutable release authority.

If the candidate is exactly identical to an existing release, the registry may report an idempotent already-published result.

---

# 17. Lockfile operating modes

Archon should support explicit reproducibility controls.

## Default mode

Archon may create/update `Archon.lock` when required.

## `--locked`

```text
Archon.lock must already satisfy the requested operation.
```

If resolution would modify it, fail.

## `--offline`

Do not require non-local acquisition.

For the initial local-only registry, this is naturally satisfied for local releases, but the flag should exist with future-compatible semantics.

## `--frozen`

Equivalent in policy to:

```text
--locked
+
--offline
```

No lockfile change and no external acquisition.

These controls belong to Archon, not Telos.

---

# 18. `archon artifact`

Archon should expose package-aware artifact operations.

Initial commands:

```text
archon artifact build
archon artifact inspect
archon artifact verify
archon artifact path
```

---

## 18.1 `artifact build`

Build/freeze the current package compiler artifact using ordinary package resolution.

Example:

```bash
archon artifact build
```

This is project-aware.

The lower-level equivalent is some form of:

```text
telos compile + explicit resolved plan + artifact output
```

---

## 18.2 `artifact inspect`

Examples:

```bash
archon artifact inspect
archon artifact inspect /path/to/foo.telosc
```

When no explicit file is supplied, Archon may resolve the current package's selected build artifact.

It then delegates low-level artifact interpretation to Telos/compiler APIs.

---

## 18.3 `artifact verify`

Verifies a package artifact.

When invoked through Archon, the command may provide stronger external authority than direct `telos verify`, because Archon can supply:

```text
resolved package release
registry receipt
artifact digest
toolchain compatibility
```

---

## 18.4 `artifact path`

Shows the currently selected artifact/build object for a package.

This is diagnostic convenience.

The path is not semantic identity.

---

# 19. `archon cache`

Cache management belongs entirely to Archon.

Initial commands:

```text
archon cache status
archon cache path
archon cache gc
archon cache clean
```

---

## 19.1 `cache status`

Reports useful cache information such as:

```text
global cache location
object count
approximate bytes
protected/pinned bytes where tracked
corrupt objects detected, if verification was requested
```

---

## 19.2 `cache path`

Displays the platform-specific global cache root.

This avoids requiring users/scripts to hardcode OS-dependent paths.

---

## 19.3 `cache gc`

Removes unused/reclaimable immutable cache objects according to retention policy.

It MUST NOT remove:

```text
local registry releases
installed toolchains
project source
```

If toolchains or registries internally share global CAS bytes, their live references must protect the corresponding objects.

---

## 19.4 `cache clean`

Explicitly removes reconstructable global cache state.

It remains distinct from:

```bash
archon clean
```

which is project-local.

---

# 20. `archon clean`

Removes project-local Archon state.

Default conceptual target:

```text
.archon/
```

or its generated build/cache/state subtrees.

It MUST NOT delete:

```text
Archon.toml
Archon.lock
source
local registry
global registry data
installed toolchains
```

Future selective options may include profile/target cleaning.

---

# 21. `archon registry`

Registry administration and inspection are Archon responsibilities.

Initial local-registry commands should include:

```text
archon registry list
archon registry show <registry>
archon registry path <registry>
archon registry verify <registry>
```

The first implementation returns at least:

```text
local
```

---

## 21.1 `registry list`

Lists configured/available registry identities.

Initially:

```text
local
```

---

## 21.2 `registry show local`

Displays registry metadata such as:

```text
identity
type
schema
path/location
package/release counts
```

---

## 21.3 `registry path local`

Prints the concrete local-registry storage root.

This command is particularly useful during bootstrap/toolchain development.

---

## 21.4 `registry verify local`

Validates local registry integrity:

```text
schema
index/release agreement
release metadata
source digests
missing source snapshots
artifact references where applicable
```

It must not silently repair immutable release authority unless an explicit future repair operation is designed.

---

# 22. `archon package`

Package inspection commands should operate over registries.

Initial forms:

```text
archon package list
archon package show <name>
```

Examples:

```bash
archon package list
archon package show parse
```

`package show` may report:

```text
available versions
registry
source digests
dependency requirements
cached compatible artifacts
```

This is registry/package-manager inspection, not compiler-artifact inspection.

---

# 23. `archon toolchain`

Archon owns installed toolchain management.

Initial commands:

```text
archon toolchain list
archon toolchain show
archon toolchain use
archon toolchain install
```

The first implementation may support only local/manual toolchain installation paths while preserving these semantics.

---

## 23.1 `toolchain list`

Shows installed toolchains.

Example:

```text
stable
dev *
```

where `*` denotes the active selection.

---

## 23.2 `toolchain show`

Displays the active toolchain or a named toolchain:

```bash
archon toolchain show
archon toolchain show dev
```

Conceptual information:

```text
toolchain identity
Telos compiler version
Archon version
Universe package/version
Universe artifact digest
semantic/artifact compatibility
installation path
```

---

## 23.3 `toolchain use`

Selects a toolchain according to the eventual global/workspace override model.

Example:

```bash
archon toolchain use dev
```

The scope of the override must be explicit.

Do not create hidden current-directory state without documented ownership.

---

## 23.4 `toolchain install`

Initially may install from an explicit local toolchain bundle/path.

A future network distributor can extend acquisition.

Installation must validate:

```text
Telos binary
Archon compatibility
Universe artifact
toolchain manifest
```

before making the toolchain active.

---

# 24. Universe handling

For normal commands:

```text
archon check
archon build
archon run
archon test
archon bench
```

Archon resolves the exact Universe artifact from the active toolchain and supplies it to Telos.

The compiler must not search random build directories for Universe.

A dedicated future/internal command may rebuild/promote Universe, but that is toolchain/bootstrap functionality, not an ordinary package fallback.

---

# 25. `archon fmt`

Formatting is package-aware.

```bash
archon fmt
```

formats the current package according to Telos formatting rules.

Potential selection:

```bash
archon fmt --check
```

for CI.

Archon owns source discovery/package scope.

The formatter implementation may live in Telos compiler tooling.

Formatting must not require dependency resolution unless language formatting actually depends on semantic context.

---

# 26. `archon doc`

Documentation generation is package-aware.

```bash
archon doc
```

resolves the package and dependencies as required, then invokes documentation tooling.

Initial documentation semantics may be limited.

Documentation tests are explicitly **not** part of the initial testing model.

The command namespace is reserved so documentation generation can evolve independently.

---

# 27. Workspaces

The manifest/workspace model may later define multiple member packages.

Archon owns workspace selection.

Expected selectors include:

```bash
archon build -p parse
archon test -p slotmap
archon build --workspace
```

`-p/--package` selects a package from the active workspace.

`--workspace` selects all applicable members.

Telos receives already-selected resolved packages/plans.

It does not interpret workspace membership.

---

# 28. Profiles

Archon owns user-facing build profiles.

Initial expected profiles:

```text
debug
release
```

Example:

```bash
archon build --release
```

Archon normalizes profile policy into compiler-relevant configuration before invoking Telos.

Profile policy may affect:

```text
optimization
debug metadata
runtime checks/contracts
lowering
```

Only compiler-relevant settings enter the resolved build plan.

---

# 29. Command output principles

Archon is a build tool and should produce concise, scriptable output.

Default human output should emphasize:

```text
what is being resolved
what is being built
cache/reuse decisions when relevant
errors/warnings
final test/benchmark summaries
produced target locations
```

It should avoid unnecessary noise during successful incremental builds.

A verbose mode may expose:

```text
dependency resolution
artifact imports
cache hits/misses
BuildKeys
source rebuild reasons
Telos invocation details
```

This will be especially useful during compiler/artifact development.

---

# 30. Structured output

Commands useful to tooling should support machine-readable output over time.

Examples:

```text
archon tree
archon artifact inspect
archon package show
archon toolchain show
archon test results
```

A structured format such as JSON should use versioned schemas.

Human output is not a stable API by default.

---

# 31. Explainability

Archon should eventually make build decisions inspectable.

Useful future commands/options include:

```bash
archon explain build
archon build --plan
archon build -vv
```

They should answer:

```text
Why did this package rebuild?
Why was this artifact rejected?
Which exact dependency release was selected?
Why did the lockfile change?
Which Universe artifact was used?
Which cache object was reused?
```

This is particularly important in an incremental artifact-based compiler.

---

# 32. Error ownership

Archon should distinguish errors by layer.

Examples:

### Package/manifest

```text
invalid Archon.toml
unknown target
default-run points to no binary
```

### Resolution

```text
package not found
version conflict
lockfile conflict
registry unavailable
```

### Cache

```text
cache object corrupt
cache write failed
```

### Toolchain

```text
toolchain missing
Universe artifact incompatible
```

### Compiler

```text
Telos semantic diagnostic
artifact import failure
lowering error
```

The outer CLI may present a unified error experience, but it should preserve diagnostic category and underlying cause.

---

# 33. Exit behavior

Exact numeric exit codes may be finalized later, but Archon must distinguish:

```text
success
usage/configuration failure
dependency resolution failure
build/compiler failure
test failure
benchmark execution failure where appropriate
registry/publication failure
toolchain failure
internal Archon failure
```

A failing test suite should produce a nonzero status even if compilation succeeded.

---

# 34. Transactional mutation

Commands that mutate durable user state must avoid half-applied results.

This includes:

```text
archon add
archon remove
archon update
archon publish
archon toolchain install
```

Examples:

```text
manifest changed but lockfile not coherently updated
registry index updated before release is complete
toolchain marked active before Universe validates
```

are invalid partial states.

The exact filesystem transaction strategy may vary.

---

# 35. No silent authority substitution

Archon MUST NOT silently change authority to "make the build work."

Examples of forbidden behavior:

```text
locked local:parse@1.4.1 missing
    -> silently use 1.4.2

registry artifact corrupt
    -> silently use arbitrary ../parse checkout

toolchain Universe invalid
    -> silently compile some nearby Universe source

path dependency absent
    -> silently use registry package with same name
```

Archon may offer explicit recovery/update actions.

Those are separate observable decisions.

---

# 36. Interaction with the local registry

Until a web registry exists, package dependencies use the local registry by default.

The lifecycle is:

```text
Package A mutable source
    ↓
archon publish
    ↓
local:A@1.0.0

Package B:
    A = "^1.0"
    ↓
archon resolve/build
    ↓
local:A@1.0.0 selected
    ↓
cached compatible A.telosc?
    ├── yes -> import
    └── no  -> rebuild from immutable registry source
```

This is the first complete package-manager flow and should be treated as production architecture, not throwaway bootstrap logic.

---

# 37. Interaction with caches

Archon owns cache lookup and promotion.

Conceptually:

```text
Resolved package release
        ↓
BuildKey
        ↓
global index/cache lookup
        │
        ├── hit -> validate/use object
        │
        └── miss -> compile from authority
                         ↓
                    immutable result
                         ↓
                    publish to CAS
```

Telos receives selected objects.

It does not scan cache directories.

---

# 38. Interaction with Telos

The clean boundary is:

```text
Archon
    ↓
ResolvedBuildPlan
    ↓
Telos compiler APIs / telos process
    ↓
compiler results
    ↓
Archon persistence/orchestration
```

Archon MUST NOT reproduce:

```text
semantic type checking
query-product logic
artifact section validation
module semantic linking rules
```

Those remain compiler-owned.

Similarly, Telos MUST NOT reproduce:

```text
registry solving
target selection
cache GC
lockfile policy
```

---

# 39. Initial command summary

A coherent initial help surface is:

```text
archon

Project:
    new
    init

Build:
    check
    build
    run
    test
    bench
    repl
    clean

Dependencies:
    add
    remove
    update
    fetch
    tree

Publication:
    publish

Artifacts:
    artifact build
    artifact inspect
    artifact verify
    artifact path

Cache:
    cache status
    cache path
    cache gc
    cache clean

Registry:
    registry list
    registry show
    registry path
    registry verify
    package list
    package show

Toolchain:
    toolchain list
    toolchain show
    toolchain use
    toolchain install

Source tooling:
    fmt
    doc
```

The initial implementation may stage these commands, but this is the intended ownership map.

---

# 40. Core invariants

Archon must preserve:

1. **Manifest as package intent**  
   `Archon.toml` describes requested package structure and requirements.

2. **Lockfile as exact resolution authority**  
   `Archon.lock` identifies selected immutable registry releases.

3. **Path dependencies remain mutable**  
   They are not silently converted to registry releases.

4. **Registry releases remain immutable**  
   Publication does not overwrite existing version authority.

5. **Target selection is explicit and deterministic**  
   Ambiguity is diagnosed.

6. **Development overlays remain separate**  
   Tests/examples/benches do not pollute the canonical production artifact.

7. **Toolchain Universe is exact authority**  
   Normal builds do not rediscover/recompile Universe ad hoc.

8. **Caches are disposable**  
   Clearing them affects performance, not package meaning.

9. **No hidden fallback**  
   Authority changes are explicit.

10. **Archon resolves; Telos compiles**  
    The package manager never becomes a second semantic compiler.

---

# 41. User-facing mental model

A developer should be able to understand the toolchain as:

```text
Archon.toml
    says what I want

Archon.lock
    records what was selected

archon
    realizes that package world

telos
    compiles/executes the resolved world

.telosc
    freezes completed compiler authority

local registry
    stores immutable published releases

global cache
    stores reusable derived objects

.archon/
    stores mutable project-local build state
```

That is the CLI architecture the first implementation should preserve.
