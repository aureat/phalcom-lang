# 06 — Spec: command contracts for `telos` and `archon`

The toolchain specification (`docs/specs/toolchain/`) owns command syntax.
This document states only what each command MUST do with executable
sections and products so that `telos run` and `archon run` work, plus the
minimal `telos` driver this program builds. Flags shown are illustrative.

## 1. `telos` — minimal driver built in this program

No `telos` binary exists at the anchor; the only driver is `phalcom`
(`phalcom-core/bin/phalcom`), which always compiles Universe from source.
Slice X7 (`08`) adds a `telos` binary with exactly the subcommands below.
Everything else in `telos-cli.md` is out of scope here.

**XT-C1. `telos compile --emit artifact` (package).** Inputs: an explicit
Universe artifact path, explicit dependency artifact paths, and the root
package source directory. Output: the package `.telosc` with compiler
sections and a schema 2 executable section (XS-B1). It MUST:

- load every input artifact through the ARTC loader and validate the chain
  (`05` §1) before compiling;
- never parse or check dependency source, and never fall back to Universe
  source (ARTC PB-B1; the `PHALCOM_DENY_SOURCE_BOOTSTRAP` gate applies);
- not search for dependencies: a dependency named in `project.toml` (later
  `Archon.toml`) without a supplied artifact is an error, not a search.

`--emit compiler-artifact` builds today's compiler-only artifact. It is for
compiler development and cannot appear in a product chain (XM-V6).

**XT-C2. `telos compile --bootstrap-universe --emit artifact`.** Builds
`universe.telosc` with its schema 1 executable section through
`build_full_universe_telosc`. This is the only mode that reads Universe
source, and it is never entered implicitly (`telos-cli.md` §11).

**XT-C3. `telos compile --emit product`.** Inputs: the root package
artifact, its chain, and an explicit target `kind:name=entry`. Output: a
`.telosx` (`04` XP-C1…C4). No source is read.

**XT-R1. `telos run <product> --artifact <path>...`.** Loads the product,
binds each supplied file to a chain entry by digest (XP-V2), validates the
chain, materializes (XM-M0…M8) and runs (XR-1…XR-5). Every chain entry must
be supplied; Telos does not look anywhere else.

**XT-R2. Program arguments.** Arguments after `--` belong to the program
(`telos-cli.md` §7.1). How they reach the program depends on the runtime
argument surface (`09` Q4).

**XT-R3. Exit classes.** At least these are distinct: success; usage error;
artifact or product incompatible or corrupt; chain or import resolution
failure (a kind of incompatibility, reported with the failing artifact and
import); runtime execution failure; I/O failure; internal failure
(`telos-cli.md` §15).

**XT-R4. Convenience form.** `telos run --artifact <path>... --entry <module>`
builds the product in memory (XT-C3 with `kind = Bin`, `name` = the entry
module text) and runs it. It applies every check a file product gets.

**XT-I1. `telos inspect` / `telos verify`.** For a package artifact with an
executable section: print or check the dependency list, the import table
grouped by family and by owning artifact, and section sizes. For a product:
XP-I1. `verify` runs XS-V1…V12 and, when the chain is supplied, `05` §1.
Both are read-only.

## 2. `archon` — what it must do to run a target

Archon is not built in this program. When it is, `archon run` (and
`archon test` / `archon bench` for pre-overlay root packages) MUST
implement this sequence, which reuses `archon build` exactly
(`builds-artifacts-and-caches.md`, "What `archon run` means").

**XA-1. Resolve.** Produce a `ResolvedBuildPlan` (`resolved-build-plan.md`)
naming the toolchain, the pinned `universe.telosc`, every dependency's
source or artifact authority, and the selected target.

**XA-2. Executable artifacts for the whole closure.** For every package in
the target's closure, obtain a package artifact *with* a schema 2 section,
from the cache or by asking Telos to build it (XT-C1). A cached
compiler-only artifact for the same package does not satisfy a run; it has
a different digest and Archon caches it under its own key.

**XA-3. Product.** Ask Telos for the product (XT-C3) and cache it. Its
build key includes the root artifact's digest and the target, so a rebuilt
dependency (new digest, D-XL-2) produces a new product.

**XA-4. Run.** Invoke `telos run` with the product and the cache path of
every chain artifact (XT-R1). Archon never edits a product and never
substitutes a different artifact for a digest.

**XA-5. Failure attribution.** Archon maps Telos's exit classes to its own,
keeping the cause: a chain digest mismatch is a cache or build-plan defect,
not a user program failure.

**XA-6. Dependency rebuilds cascade.** Because linked sections bind exact
digests, rebuilding a dependency invalidates every dependent's executable
artifact and every product that contains it. Archon's build index MUST key
dependents on dependency digests (`builds-artifacts-and-caches.md`,
"Dependency invalidation"; ARTC Q5).
