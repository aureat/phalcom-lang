# 02 — Decisions

Each ruling states the decision, the reason, and the alternatives rejected.
Status of every ruling: **PROPOSED** until the user ratifies this spec. Once
ratified, implementers MUST NOT re-derive them.

### D-XL-1: Package bytecode lives in the package artifact

A package's bytecode is an optional `Executable` section of the package's
own `.telosc`, built in the same pass as its compiler sections and covered
by the same artifact digest.

*Why.* The toolchain requires one atomic artifact per package generation
(`docs/specs/toolchain/README.md` §7) and says target products derive from
that generation (`builds-artifacts-and-caches.md`, "Package generation
versus target product"). Library bytecode is target-independent, so it
belongs to the generation. Keeping it in the same container means one
digest covers both the compiler view and the executable view, so they
cannot drift apart.

*Rejected.*
- **Separate `.telosb` bytecode file per package.** Two files per package
  generation, two digests to bind, and a new way for them to disagree.
- **Bytecode only in the product.** Every product would re-capture every
  dependency's bytecode; nothing would be shared across targets or projects,
  and the cache would hold many copies of the same code.

### D-XL-2: Linked sections bind dependencies by exact artifact digest

A linked section lists each dependency artifact by `StableProjectRef` and
artifact digest. Loading against any other bytes is an error.

*Why.* Bytecode bakes in facts about its dependencies that no stable
identity captures: instance field slots inherited from a superclass, the
global slot layout behind each linked read after materialization, the
stack shape of inlined operations. Exact binding makes all of them correct
by construction. The toolchain already rebuilds dependents when a
dependency changes (ARTC EXP-P2 and Q5), so this costs nothing new.
Universe needs this most: its `StableProjectRef` (`Builtin { universe,
0.1.0 }`) carries no fingerprint, so without a digest two different
Universe builds are indistinguishable.

*Rejected.*
- **Per-import layout or interface fingerprints** (ABI-compatible upgrades).
  Correct in principle, but it needs a definition of executable ABI
  compatibility for every operand domain, and nothing needs it yet. It can
  be added later as a schema 3 without breaking schema 2.
- **Binding by project identity only.** Unsafe for Universe, and unsafe for
  any rebuilt package whose identity inputs did not change.

### D-XL-3: An explicit import table, closed in both directions

A linked section carries a table of every external identity it uses
(modules, bindings, classes, declarations, callables, implementations,
variants, type and kind digests). Every external reference in the section
appears in the table, and every table entry is used by at least one
reference.

*Why.* It turns cross-artifact validation into one pass over a small table
before materialization, instead of a search through every prototype and
recipe during it (XL-I6). It makes "what does this package need from
Universe" inspectable (`telos inspect`). Requiring every entry to be used
keeps the table canonical, which determinism needs (XL-I8).

*Rejected.*
- **Implicit imports (scan all rows at load).** Same check, but spread
  through materialization, so a failure can arrive after objects were
  allocated, and there is nothing to show in `inspect`.
- **Reusing the `ModuleWorldV5` external table.** That table covers
  module-world references only (ruling D-MW5-2). Executable references
  include classes, callables and type digests, which it cannot express.
  The linked section's module imports MUST agree with it (XS-V5).

### D-XL-4: Schema 2 for linked sections; schema 1 stays closed and Universe-only

Linked sections use a new `EXECUTABLE_SCHEMA_V2`. The Universe artifact
keeps writing schema 1. The header keeps one capability bit,
`PORTABLE_EXECUTABLE`, which means "has an executable section". The
section's schema says whether it is closed or linked.

*Why.* The schema check in `ExecutableImageV1` already rejects any other
schema, so a reader that predates schema 2 fails closed. A new capability
bit would not: `TeloscCapabilities::from_bits_truncate` drops unknown bits,
so an old reader would silently ignore the bit and might accept the
artifact as compiler-only. Keeping Universe on schema 1 keeps every Universe
digest and cached artifact valid (same reasoning as D-MW5-3).

*Rejected.*
- **New `LINKED_EXECUTABLE` capability bit.** Silently dropped by current
  readers, as above.
- **Moving Universe to schema 2 now.** Changes every Universe digest for no
  functional gain. Revisit when ARTF001 P5 rewrites the Universe build.

### D-XL-5: The product is a separate small file with no bytecode

A runnable target is a `.telosx` product: target kind, name, entry module,
the artifact chain by digest, and the ABI identity. All bytecode stays in
the artifacts.

*Why.* Library bytecode is shared by every target of a package and by every
project that depends on it (D-XL-1). What differs between targets is only
the entry and the dependency closure, and a few hundred bytes record that.
A product is also the unit Archon caches and hands to `telos run`, matching
"`telos run` executes what has already been selected" (`telos-cli.md` §7).

*Rejected.*
- **No product; `telos run --entry M --artifact ...` only.** Kept as a
  convenience form (XT-R4), but it is not a reproducible artifact, and
  Archon needs something to cache and to verify against.
- **Self-contained bundle as the only form.** Duplicates every dependency
  per product. Left as a future product option (XP-W7).

### D-XL-6: The product names artifacts by digest, never by path

The caller supplies a path for each artifact at run time. Telos checks
each file's digest against the product and never searches directories.

*Why.* The toolchain forbids Telos from owning cache layout
(`telos-cli.md` §18; `resolved-build-plan.md`, "No cache policy inside
Telos"). A path baked into a product would break on every other machine and
would let a stale file stand in for the right one.

*Rejected.*
- **Relative path hints in the product.** Would invite Telos to search.

### D-XL-7: Capture compiles in a VM bootstrapped from the dependency chain, and does not run package code

A package's bytecode is captured by bootstrapping a VM from the dependency
chain's artifacts (Universe executable plus earlier packages' linked
sections), materializing the package's linked program without running it,
emitting each owned module's closure with the ordinary compiler, and
capturing those closures.

*Why.* This is the same emitter and heap-object path that `run_compiled`
uses today, so the bytecode is the bytecode the source path runs (XL-I7).
Bootstrapping from the dependency artifacts, not from dependency source,
keeps the build source-free for dependencies (ARTC PB-B1). Not running
package initializers keeps the build free of program side effects.

*Rejected.*
- **Capturing after running the program.** Builds would execute arbitrary
  package code; captured constants could reflect run-time state.
- **A new emitter that writes portable instructions directly.** A second
  code generator that must match the first forever.
- **Bootstrapping the capture VM from Universe source.** Defeats source-free
  builds and the `PHALCOM_DENY_SOURCE_BOOTSTRAP` gate.

### D-XL-8: Packages carry no natives and no primordial roles

Schema 2 has no `native_anchors` or `primordial_roles` tables at all, so a
package cannot express them. `bootstrap.native_runtime_roles` MUST be
empty. The section's compatibility block carries the native ABI digest of
the Universe it was compiled against, copied rather than recomputed, and
loading checks it against the loaded Universe.

*Why.* Natives are bound only by the Rust runtime, against Universe
classes. A package that names a native anchor could claim a primitive it
does not own.

### D-XL-9: Which package modules run is decided per product, not per package

A linked section marks no module eager. The product records the
initialization order of package modules for its entry (`04` XP-W8). This
refines D-MW5-4: "the dependency initializes completely before the
importer" becomes "the part of the dependency in the product's
initialization closure initializes before any of the importer's part".

*Why.* Which modules a program initializes depends on its entry. `slotmap`
declares `entry = "slotmap.main"`. When `slotmap-runtime-tests` depends on
`slotmap`, the source path initializes only the `slotmap` modules the tests
reach, never `slotmap.main`. If the `slotmap` artifact marked its own entry
eager, every program that depended on it would run SlotMap's `main` as a
side effect. The source path already computes the order per program
(`CompiledProgram::initialization_order`); the product keeps that property.
Universe is unaffected: its bootstrap plan stays as it is.

`ModuleWorldV5` already stores an `entry` and an owned
`initialization_order` per artifact. Those describe the package's own
declared entry program. They remain valid compiler data and serve as a
cross-check (XP-C4), but they never decide what runs when the package is
used as a library.

*Rejected.*
- **Eager set per package artifact (as Universe has).** Wrong for any
  package that has an entry and is also used as a library.
- **Computing the order at run time.** Possible, but then `telos run` must
  re-derive linker policy, and the order is not visible in `inspect` or
  covered by the product digest.
