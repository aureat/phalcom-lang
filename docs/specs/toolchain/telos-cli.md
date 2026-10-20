# `telos` Command-Line Interface

This specification defines the initial command-line interface of `telos`, the Telos compiler/runtime driver.

`telos` is intentionally narrower than `archon`.

Its governing rule is:

> **`telos` realizes an already-resolved Telos compilation or execution request. It does not decide what package graph, package version, registry release, target, or toolchain the user intended.**

`archon` is the ordinary project/package tool. Most users should invoke `telos` indirectly through Archon.

`telos` remains first-class because compiler developers, build-system integrations, artifact tools, test infrastructure, and advanced users need direct access to language realization.

---

# 1. Responsibility boundary

`telos` owns:

```text
source parsing
module loading from explicit source authority
module linking
semantic analysis
semantic query/product realization
compiler-artifact import
compiler-artifact export
artifact validation
target lowering
runtime execution
REPL execution
artifact inspection
```

`telos` does not own:

```text
Archon.toml dependency solving
Archon.lock mutation
registry search
package publication
toolchain installation
cache garbage collection
workspace package selection
default binary selection
version requirement resolution
"find newest artifact" behavior
```

A useful boundary test is:

> If a decision requires choosing among multiple package, registry, target, or toolchain possibilities, it belongs to Archon before `telos` is invoked.

---

# 2. Initial command surface

The initial public command families are:

```text
telos compile
telos check
telos run
telos repl
telos inspect
telos verify
```

These commands may share internal compiler APIs. Their CLI semantics remain distinct.

The expected usage split is:

```text
archon build     -> project-aware orchestration
telos compile    -> explicit compiler operation

archon run       -> resolve/build/select target, then execute
telos run        -> execute an already-resolved runnable product

archon artifact inspect ...
telos inspect ...
```

---

# 3. Input model

Direct `telos` invocation must use explicit compiler inputs.

The preferred long-term architecture is that Archon constructs a resolved compilation request equivalent to the `ResolvedBuildPlan` specification and passes the relevant compiler-facing information to Telos.

The CLI MAY initially expose explicit flags rather than one serialized build-plan file.

Whichever representation is used, the semantics are the same:

```text
exact root package/source authority
exact dependency authorities
exact Universe authority
exact selected target where relevant
exact compile mode/profile
exact requested outputs
```

By the time Telos executes:

> No dependency version requirement remains unresolved.

Telos should never receive:

```text
parse = "^1.4"
```

and be expected to choose a release.

It should receive an exact resolved authority such as:

```text
local:parse@1.4.1
artifact digest A
```

or an explicit source authority.

---

# 4. Explicit source and artifact authority

Telos must distinguish these operations:

```text
compile this source authority
```

and:

```text
import this artifact authority
```

It MUST NOT implement implicit:

```text
try artifact
if broken:
    search for source
    compile source
```

If an artifact supplied to Telos is corrupt, incompatible, or unauthorized, the command fails.

Archon may decide to construct a later explicit source-compilation action.

That is an orchestration decision, not hidden compiler fallback.

---

# 5. `telos compile`

`telos compile` is the primary low-level compiler command.

Conceptually:

```bash
telos compile [explicit resolved inputs] [output requests]
```

It may compile:

- a package from source;
- a package against imported dependency artifacts;
- a selected executable target;
- a development overlay supplied by Archon;
- Universe in explicit bootstrap mode.

It MUST NOT discover registry dependencies by itself.

---

## 5.1 Compilation inputs

The command must be able to receive or derive from an explicit resolved-plan input:

```text
root package identity
root source authority
resolved dependency graph
Universe authority
source/artifact realization for every dependency
compile configuration
selected target
output requests
```

An implementation may expose this through a structured plan file, IPC, or explicit arguments.

The stable semantic requirement is more important than the first syntax.

---

## 5.2 Example direct compilation

A conceptual direct invocation could look like:

```bash
telos compile \
  --project /work/atlas \
  --universe /toolchains/dev/lib/universe.telosc \
  --emit artifact \
  --output /tmp/atlas.telosc
```

This example is illustrative.

A production Archon integration should normally use a richer typed/structured interface so exact dependency authority is not encoded through fragile ad hoc path conventions.

---

## 5.3 Package artifact output

When asked to freeze a package artifact:

```text
completed semantic generation
        ↓
encode .telosc
        ↓
validate
        ↓
return artifact/result to Archon
```

Telos owns:

```text
artifact format
section encoding
semantic projection
query image projection
cross-section validation
compatibility validation
artifact digest computation where compiler-owned
```

Archon owns:

```text
global cache placement
build-index update
registry association
promotion policy
garbage collection
```

Telos MUST NOT need to know the global Archon cache directory.

---

## 5.4 Executable output

A binary target is specified explicitly.

Conceptually:

```text
target:
    kind = bin
    name = server
    entry = atlas.server
```

Telos performs target-specific executable/runtime lowering from the package semantic generation.

It does not read `default-run` to select the target.

Archon has already resolved that choice.

---

## 5.5 Development target output

For test/example/bench operations, Telos receives an explicit development overlay.

It must preserve:

```text
base production package generation
+
development overlay
```

rather than merging development declarations into the canonical production package artifact.

The initial test mode is:

```text
integration test
Archon-managed harness
```

There is no direct compiler mode for:

```text
unit-test declarations inside production modules
documentation tests
harness-free tests
```

because those features are not part of the initial package model.

---

# 6. `telos check`

`telos check` performs semantic compilation without requiring final executable output.

Its role is:

```text
parse/link/analyze
realize required semantic authority
report diagnostics
avoid unnecessary final lowering
```

It may still import `.telosc` dependencies.

It may still update a caller-owned persistent compiler session when invoked through a long-lived integration.

It does not imply publication of a package artifact.

---

## 6.1 Direct versus Archon use

Direct:

```bash
telos check ...
```

means:

> Check these explicit inputs.

Archon:

```bash
archon check
```

means:

> Resolve this package/workspace, dependencies, toolchain, cache state, and check scope, then invoke the compiler.

They are related but not equivalent command semantics.

---

# 7. `telos run`

`telos run` executes an already-resolved runnable Telos product.

It should not mean:

> Open `Archon.toml`, choose a binary, build stale dependencies, and execute it.

That is `archon run`.

The low-level command may support forms such as:

```bash
telos run <compiled-product>
```

or an explicit resolved compilation-and-run mode supplied by Archon.

The exact executable file format may evolve.

The architectural rule is stable:

> `telos run` executes what has already been selected; it does not perform package-manager target selection.

---

## 7.1 Program arguments

The CLI should follow the conventional separator:

```bash
telos run product -- arg1 arg2
```

Arguments after `--` belong to the Telos program.

Arguments before `--` belong to the `telos` driver.

---

## 7.2 Runtime exit status

For a successfully launched Telos program, the program's intended process status should be preserved where the runtime model permits it.

Compiler/driver failures must remain distinguishable from program exit status.

The command should not collapse:

```text
program returned nonzero
```

and:

```text
compiler artifact could not be loaded
```

into the same diagnostic category.

---

# 8. `telos repl`

`telos repl` provides a compiler/runtime interactive environment.

Two environments are useful.

## Standalone REPL

```bash
telos repl
```

uses an explicitly configured/default standalone compiler environment.

Its package capabilities may be intentionally limited.

## Package-aware REPL through Archon

```bash
archon repl
```

should be the normal way to start a REPL with:

```text
current package
resolved dependencies
toolchain Universe
package imports
development configuration
```

Archon resolves the environment.

Telos hosts the actual interactive language session.

The REPL should use the same semantic/compiler model as ordinary compilation rather than a second evaluator with incompatible semantics.

---

# 9. `telos inspect`

`telos inspect` is the low-level compiler artifact viewer.

It should work directly on `.telosc` files.

Initial forms should support:

```bash
telos inspect package.telosc
telos inspect package.telosc header
telos inspect package.telosc modules
telos inspect package.telosc semantic
telos inspect package.telosc queries
telos inspect package.telosc metadata
```

Additional sections may be added as the artifact format evolves.

---

## 9.1 Default inspection output

Without a section:

```bash
telos inspect foo.telosc
```

should show a concise high-level summary such as:

```text
Package: foo@1.2.0
Container schema: 2
Semantic model: ...
Artifact digest: ...
Capabilities:
  compiler-image
Modules: ...
Queries: ...
Metadata: ...
```

This output is diagnostic/presentation, not a stable machine protocol unless explicitly documented.

---

## 9.2 Structured output

Inspection should eventually support machine-readable output, for example:

```bash
telos inspect foo.telosc --format json
```

Machine-readable schemas should be versioned.

Human output may evolve for usability.

The CLI MUST NOT make scripts parse decorative human output as the only supported machine interface.

---

## 9.3 Semantic visibility

Artifact inspection may reveal internal compiler rows for debugging.

That does not change Telos language visibility.

The inspector is a tooling/debugging operation.

It MUST NOT be interpreted as proof that downstream package code may import or access every declaration shown in the artifact.

---

# 10. `telos verify`

`telos verify` validates a compiler artifact without compiling a consuming package.

Example:

```bash
telos verify foo.telosc
```

The verification path should include all format-local checks available without external package-resolution context.

Depending on artifact architecture, these include:

```text
container magic/schema
section directory integrity
size/allocation limits
section decoding
declared digests
artifact digest
cross-section references
stable identity well-formedness
semantic/query image consistency
capability validity
```

Where external package authority is required, the verifier must distinguish:

```text
artifact internally valid
```

from:

```text
artifact externally authorized for resolved package release X
```

The latter requires resolver evidence supplied by Archon or another trusted caller.

---

## 10.1 Verification must be read-only

`telos verify` MUST NOT:

```text
rewrite artifact
repair corrupt sections
replace it from cache
fetch source
compile source
```

It is a validation operation.

---

# 11. Universe bootstrap mode

Normal Telos package compilation receives an artifact-backed Universe authority.

Rebuilding Universe requires an explicit bootstrap mode.

The CLI or compiler API must make the distinction impossible to miss.

Conceptually:

```bash
telos compile \
  --bootstrap-universe \
  --project packages/universe \
  --emit artifact
```

Exact syntax is deferred.

The semantic requirements are:

```text
do not import installed Universe as the same canonical Universe authority
retain canonical ProjectIdentity::Universe
compile packages/universe source through the dedicated bootstrap path
produce candidate universe.telosc
```

Normal package compilation MUST NOT accidentally enter this mode because an installed Universe artifact is missing.

---

# 12. Diagnostics

Telos owns language/compiler diagnostics.

Examples:

```text
syntax error
unresolved import after resolved project graph construction
type mismatch
access violation
non-exhaustive match
artifact ABI mismatch
artifact corruption
runtime compilation/lowering error
```

Archon owns orchestration diagnostics such as:

```text
package version not found
multiple binaries are available
local registry cannot be opened
lockfile would change under --locked
```

When Archon invokes Telos, compiler diagnostics should be surfaced without losing structured source/package identity.

---

# 13. Standard output and standard error

The CLI should preserve conventional stream discipline.

### `stdout`

Use for requested program/tool output.

Examples:

```text
program stdout
machine-readable inspect output when requested
explicit artifact data only if a command deliberately streams it
```

### `stderr`

Use for:

```text
human diagnostics
compiler errors
warnings
driver status messages
```

A successful program's stdout must not be polluted by build-driver chatter if Archon/Telos need to preserve scriptability.

---

# 14. Machine-readable diagnostics

Telos should support a structured diagnostic mode for Archon, IDEs, and other tooling.

Conceptually:

```bash
telos check --diagnostic-format json
```

The exact protocol should include stable fields such as:

```text
severity
diagnostic code
message
stable source identity
source range
related diagnostics
notes/fixes where supported
```

Archon should consume structured diagnostics rather than scraping terminal text.

---

# 15. Exit status classes

Exact numeric assignments may be finalized by implementation, but the CLI must distinguish at least:

```text
success
usage/argument error
source compilation failure
artifact incompatibility/corruption
runtime execution failure
I/O/input failure
internal compiler failure
```

For direct `telos run`, an ordinary program exit should remain distinguishable from compiler-driver failure.

Archon may map these classes into its own process-exit policy while preserving diagnostic cause.

---

# 16. `telos` does not own `Archon.toml`

Direct Telos tooling may accept a manifest path as a convenient way to identify explicit source/module metadata during migration or bootstrapping.

That MUST NOT become an architectural dependency where Telos:

```text
reads dependency version requirements
selects registry releases
writes Archon.lock
decides dev dependency inclusion
chooses default-run
```

The long-term compiler-facing contract is the normalized resolved build plan.

---

# 17. `telos` does not own registry I/O

Telos MUST NOT:

```text
search package indexes
download package releases
choose a registry
publish packages
yank package versions
authenticate to registries
```

Artifact import may consume trusted package-resolution receipts.

Those receipts are supplied by Archon or another trusted embedding layer.

---

# 18. `telos` does not own global cache policy

Telos may:

```text
encode deterministic artifact bytes
compute/verify artifact digests
consume explicit artifact paths/handles
```

It MUST NOT own:

```text
~/.cache/archon
cache eviction
cache GC
BuildKey indexes
download cache
registry object promotion
```

This prevents compiler semantics from becoming dependent on one package manager's storage layout.

---

# 19. `telos` does not own toolchain installation

The running Telos compiler has intrinsic compatibility information.

But selecting/installing:

```text
stable
nightly
custom toolchain X
```

is an Archon/toolchain-manager decision.

Direct Telos invocation may expose:

```bash
telos --version
```

and compatibility details.

It should not become the toolchain installer.

---

# 20. Version reporting

`telos --version` should report enough identity to diagnose artifact compatibility.

Conceptually:

```text
telos 0.8.0
semantic-model 12
artifact-schema 2
semantic-artifact-abi 2
query-fingerprint-abi 2
native-surface <fingerprint>
```

Human output can be concise.

A structured form should eventually exist for Archon/toolchain integration.

---

# 21. Direct compiler use is intentionally advanced

A user should not need to manually construct:

```text
Universe path
dependency artifact graph
package authority receipts
output policy
```

for ordinary development.

That is why:

```bash
archon build
```

exists.

Direct `telos compile` is analogous to using a compiler driver with explicit resolved inputs.

It remains powerful, but Archon is the ergonomic project layer.

---

# 22. No hidden environment-dependent semantics

Telos should avoid making semantic compilation depend on ambient state that is not represented in its explicit inputs.

For example, it should not silently inspect:

```text
current directory for a random package
environment package search paths
global registry state
"latest" cached artifact
unrelated Archon.lock
```

If ambient configuration materially affects compilation, Archon must normalize it into explicit plan/configuration authority.

This is necessary for reproducibility.

---

# 23. Initial command synopsis

The initial intended surface is:

```text
telos [global-options] <command>

Commands:
    compile     Compile explicit resolved source/artifact inputs
    check       Perform semantic compilation without required final emission
    run         Execute an already-resolved runnable Telos product
    repl        Start a Telos interactive compiler/runtime session
    inspect     Inspect a .telosc compiler artifact
    verify      Validate a .telosc compiler artifact

Global:
    --version
    --help
```

Additional compiler-debug commands should be added only when their ownership is clearly language/compiler-level rather than project-management-level.

---

# 24. Core invariants

The implementation must preserve:

1. **No package solving in Telos**  
   Version requirements are resolved before invocation.

2. **No registry I/O in Telos**  
   Registry authority is supplied externally.

3. **No target ambiguity in Telos**  
   The selected target is explicit.

4. **No cache policy in Telos**  
   Artifact locations/handles are supplied by the caller.

5. **Explicit source versus artifact realization**  
   Artifact failure never silently becomes source compilation.

6. **Fail-closed artifact validation**  
   Corruption and incompatibility are errors.

7. **One compiler semantic model**  
   Imported wire data reconstructs current compiler products.

8. **Same compiler semantics across CLI and embedding**  
   Direct CLI, Archon integration, REPL, and tests should reuse the same core compiler architecture.

9. **Artifact determinism**  
   Compiler output should not depend on incidental host/process state.

10. **Universe bootstrap is explicit**  
    Normal builds and Universe rebuilds cannot be confused.

---

# 25. Relationship to Archon

The most important interaction can be summarized as:

```text
ARCHON
    asks:
        what package?
        what versions?
        what registry?
        what toolchain?
        what Universe?
        what target?
        what profile?
        what cached objects?

        ↓

ResolvedBuildPlan

        ↓

TELOS
    asks:
        are these inputs valid?
        what semantic authority follows?
        what compiler products follow?
        what artifact/executable should be emitted?
```

The CLI boundary is successful when Telos never needs to answer an Archon question in order to compile.
