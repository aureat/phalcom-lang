# `universe.random` implementation walkthrough at delivery stop

Status: IN_PROGRESS. Completion: PARTIAL. Verification: FOCUSED_TESTED.
Branch: `main`. Starting HEAD: `800474d8`; stable owner prerequisite: `fbf057ea`.
User requested delivery after the focused negative/resource gate. No subagents
were used. The delivery commit contains this record and the implementation;
retrieve its revision with `git log --oneline --grep='random'`.

## Delivered architecture

`universe.random` is registered as a package with private child modules for
`Generator`, `Xoshiro256PlusPlus`, `Distribution`, `UniformInt`, `Bernoulli`,
and `SecureRandom`. The public facade exports the typed deterministic API and
secure capability. Deterministic behavior is ordinary source code. Xoshiro
state is explicit mutable class state; `nextU64` is the only inherent witness
operation and the trait supplies defaults for booleans, floats, arbitrary-size
unbiased integers, bytes, choice, and Fisher–Yates shuffle.

The stream uses standard SplitMix64 increment-before-mix seeding and standard
xoshiro256++ transition. The exact choices are: high bit for bool; top 53 bits
over 2^53 for float; earlier words as high chunks with excess high bits masked;
big-endian byte extraction; no reservoirs; descending shuffle; zero consumption
for bound one and Bernoulli endpoints.

`UniformInt` and `Bernoulli` are validated immutable descriptors with associated
`Output` types. `SecureRandom` is separate and does not implement `Generator`.
Its only native operation fills an existing `Bytes` buffer using `getrandom`
0.3.3; source code owns allocation and raises `EntropyError` on host failure.
The native implementation releases its mutable heap borrow before allocating an
error String. Effects are mutation + nondeterminism + blocking.

## General prerequisite repairs

Before random completion, stable callable metadata was upgraded to preserve
Declaration/Variant/Conformance owners with schema v11 and v10 legacy decode.
Method-generic trait evidence now translates requirement binders to selected
witness binders and carries the existing call-entry recipe through selected
trait lowering and VM activation. Trait slots are local to their owning surface.
Qualified module class exports use canonical type-name resolution. These are
general semantic/runtime seams; no random-specific opcode or dispatch branch was
added.

## Acceptance evidence

- Original syntax probe and deterministic random corpus: PASS, 2 selected.
- Core random facade/evidence, secure runtime, negative typing/topology and
  numeric-policy tests: PASS, 4 selected (final gate).
- Core trait suite: PASS, 18 selected.
- Core secure native unit tests: PASS, 2 selected.
- Semantic source Universe constants/trait-default publication: PASS.
- Semantic selected-witness binder/proof and cold/incremental parity regressions:
  PASS.
- `cargo check -p phalcom-modules`, `-p phalcom-native-meta`,
  `-p phalcom-native-surface`, and `-p phalcom-core`: PASS.
- Native surface generator `--check`: PASS, 400 declarations.
- `git diff --check`: PASS.

The full semantic trait lane remains 41 PASS / 4 baseline failures. The four
existing failures are recorded in the incident and were reproduced with the
random/recipe changes isolated; they are outside random acceptance.

Workspace release certification, broad core tests, performance benchmarking,
dedicated effect/native-source identity assertions, remaining byte resource
cases, and cross-platform execution were not run. The focused injected native
failure test executes the public `bytes` API and catches source `EntropyError`;
the native zero/nonempty success tests also pass. Full workspace formatting
reports baseline drift; only scoped Rust files were formatted.

## Last completed step and remaining acceptance

The final core random lane initially failed because its error-text expectations
used the wrong case and hidden-import wording. The assertions were corrected to
the actual diagnostics (`GenericConstraintUnsatisfied`, `ModuleExposureRejected`)
without changing rejection behavior. The final run passed 4/4, including the
70-bit candidate assembly and oversized-bound numeric failure.

Next: record the required throughput/allocation/GC measurements; complete the
metadata/effect assertions and byte-resource tests; run directly affected provider,
native verification, numeric and full corpus gates; decide/run broad core gate;
then promote plan completion only from completed evidence. The source slice is
implemented, but the plan remains PARTIAL at the user-directed stop.

The retained corpus session separately exposed an existing update failure:
`FieldSignature(Xoshiro256PlusPlus._s0)` was Ready but not validated for the new
revision, and last-known-good fallback returned stale fixture source. The syntax
probe now has its own lane. This general dependency-validation issue remains
recorded, not repaired by the fixture isolation.

## Deferred by plan

Jump/longJump, stream splitting, parallel reproducibility, complex distributions,
weighted/reservoir sampling, state serialization, secure token/UUID helpers,
fixed-width optimization, and mutation-capability integration remain deferred.
