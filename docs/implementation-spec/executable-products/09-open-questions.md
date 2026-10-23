# 09 — Open questions and risks

None of these blocks slices X0–X2. Items marked **BLOCKING-IF** stop the
program at the slice named when their condition is found true.

## Q1. File extension and command names

`.telosx` and `telos compile --emit product` are provisional. The toolchain
specification owns names (`telos-cli.md` §23). Renaming changes no rule in
this spec.

## Q2. Package-owned conformances on foreign classes

XS-V8 rejects any method recipe held by a foreign class. If the language
lets a package implement its own trait for a Universe class (for example
`impl Describe for Int`), and lowering installs that as a method on `Int`,
then XS-V8 rejects a legitimate program. PDR-0001 (classes are closed)
suggests this cannot happen through class reopening, but trait conformance
is a separate path. **BLOCKING-IF** census item X0.2 finds such an
installation: a ruling is needed on where package-owned conformance rows
live (a package-owned dispatch overlay versus installation into the foreign
class at materialization), and XL-I4 must be restated.

## Q3. Type digest ambiguity across pools

XM-V4 resolves imported type and kind digests by first match in chain
order, as `canonical_type` does today. If two pools can hold rows with the
same structural fingerprint but different runtime meaning, first-match is
wrong. **BLOCKING-IF** census item X0.4 finds an ambiguous digest for
SlotMap: imports then need `(owning artifact, digest)` pairs instead of bare
digests, which is a wire change to XS-W3.

## Q4. Program arguments and exit status

At the anchor, programs have no way to read their arguments or request an
exit status: `phalcom-core/src/primitive/system.rs` has no such primitive.
Driver exit codes exist (`ExitCode` in `phalcom-core/src/interpret.rs`:
success, usage, compile error, runtime error, no input, I/O error), and
`telos` reuses them, adding an artifact-incompatibility class (XT-R3).
Until a program surface exists, `telos run` accepts `--` arguments and does
not pass them on, and the exit status reflects only success or runtime
failure. Defining the surface is a
language and library decision outside this program.

## Q5. Cost of linear type-digest scans

`canonical_type` scans every pool and every type row per lookup. Adding a
pool per package multiplies that work at materialization time. Acceptable
for SlotMap; measure before claiming any startup number (perf numbers come
only from `perf-log/SCOREBOARD.md`). An index by digest is a local
optimization that changes no rule here.

## Q6. Universe deferred modules reached from packages

A package module that reads a binding of a deferred Universe module relies
on the existing on-demand initialization path and the V4 availability
witnesses (ARTC MW5-V9, LD-W1). This path is exercised today only by
source-compiled user programs. VER-X4 covers SlotMap; a dedicated fixture
that reads a deferred Universe module first from a package initializer
SHOULD be added in slice X6 if the census shows SlotMap does not reach one.

## Q7. Development overlays

Archon's test, example and bench overlays compile development modules over
a production package generation without changing it. Their bytecode has no
home in this spec: it is neither in the production artifact nor in a
product (products hold no bytecode, D-XL-5). The likely answer is an overlay
artifact (a package-like unit with its own identity and a schema 2
section), but that needs the toolchain's overlay identity rules first.
Until then, test suites are ordinary root packages (01 §7).

## Q8. ABI-compatible upgrades

D-XL-2 binds exact digests, so any Universe rebuild invalidates every
package executable artifact, even when nothing visible to packages changed.
For a toolchain that ships Universe updates often, this will matter. A
future schema 3 could replace exact digests with per-import interface
fingerprints. Not needed for the SlotMap goal.

## Q9. Source-path initialization order

`RuntimeGraph::initialization_order` (`phalcom-modules/src/graph.rs`)
always takes the smallest ready `ModuleId`. `ModuleId` orders by
`ProjectIdentity` first, and `ResolvedProjectId` is a per-session number,
so the source path:

- interleaves modules of different packages whenever both are ready,
  so a source run does not follow the dependency-first rule that D-MW5-4
  sets for artifacts;
- breaks ties by an order that depends on how the session numbered
  projects, not on any stable identity.

Products use XP-C4 instead. Aligning the source path (group by project in
dependency order, tie-break by stable module identity) is a follow-up. It
changes the order in which source programs run initializers, so it needs
its own ruling and a check of the golden corpus. Until then, parity tests
avoid programs whose output depends on that order (XR-5).

## Risk R1. Shared code path with every source run

XM-M3 refactors `materialize_program`, which every `phalcom` run and every
REPL evaluation goes through. Regressions there are not artifact bugs and
will show up far from this work. VER-XR4 (full `core` target) is the guard;
the REPL test target SHOULD also run at X3's gate.

## Risk R2. Capture of emitter-embedded singletons

The emitter may embed Universe singletons or interned objects as constants
in ways that the Universe-only capture never had to classify, because
inside Universe everything is owned. XS-C3 makes these errors by default.
Expect census item X0.1 to find some, and expect each to need either an
existing reference recipe or a new `ConstantRecipeV1` variant. A new
variant changes schema 1's row type too, so it MUST be added in a way that
leaves Universe bytes unchanged (append-only tag, unused by Universe).
