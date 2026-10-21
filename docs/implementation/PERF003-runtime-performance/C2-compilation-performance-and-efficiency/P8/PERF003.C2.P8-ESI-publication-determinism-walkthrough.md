---
id: PERF003.C2.P8
kind: walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
date: 2026-10-21
scope: ESI capture, semantic projection, and deterministic structural publication
---

# PERF003.C2.P8 ESI publication-determinism walkthrough

## Scope and outcome

This records the completed capture/projection/determinism slice of active plan
`PERF003.C2.P8`. It is not the final P8 walkthrough. P8 remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; executable validation and runtime
acceptance remain open.

## Implemented

- Added portable executable DTOs for compatibility, instructions, constants,
  modules, prototypes, semantic pools, images and non-authoritative staging.
- Added bounded codec/structural validation and the exhaustive 128-opcode
  manifest. The persistence census records every current bytecode, heap object,
  constant recipe, callable/chunk field and semantic-pool vector family.
- Captured canonical Universe compiler/emitter products in a disposable
  source-built VM before running each captured initializer. The captured image
  includes module and nested callable prototypes, constants, semantic pools,
  class recipes, native anchors, primordial roles, source maps and bootstrap
  disposition.
- Corrected data-construction type projection to use the exact attached
  metadata bundle root rather than resolving a pool-local placeholder against
  unrelated runtime pool state.
- Canonicalized portable type/kind recipe publication after projection. Global
  unions use stable member order while retaining repeats and cardinality;
  scoped unions and ordered type constructors keep their semantic order. Final
  recipe IDs and references are rewritten from stable keys.
- Kept executable section staging non-authoritative: section tag 7/schema 1
  can be written only while `PORTABLE_EXECUTABLE` is clear.

## Consultation/ruling outcome

The user's ruling required observing the generic `Interval<T>` semantic input,
metadata root and child fingerprints before changing fingerprint production.
The discriminator showed stable semantic inputs and exact metadata roots, but a
pool-0 runtime coordinate resolved to the wrong root. The correction was made
at that root-resolution owner. Metadata schema v14 and executable schema v1
were retained. ESI publication ordering then proceeded under the separately
approved semantic-order/publication split.

## Evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib`: PASS.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test executable_image_codec_test`: PASS, 9 tests.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib artifact::semantic:: -- --nocapture`: PASS, 4 tests.
- The focused canonical capture integration test passed in two independent
  processes. Both emitted 4,999,601 bytes with digest
  `c2af943406376eae636979f1223ea58895db3265a2d43dcf48c5da759267efea`; each
  structurally decoded and round-tripped the payload.
- The opcode manifest test passed 1/1. The staging-capability rejection passed
  1/1. Formatting and `git diff --check` passed.

The focused commands above are recorded evidence from the implementation pass;
they were not rerun while preparing this lifecycle update.

## Deferred work

No executable semantic validator, validated-image authority, VM relocation or
source-free execution path is established. P8.G2–G5 and G7 remain unrun or
deferred. P8.G6 is partial: the capture-payload determinism discriminator
passed; full artifact determinism, failed-load retry and separate startup
timings remain open. Workspace/release gates were not run. The final P8
walkthrough and handoff must be written only after those acceptance decisions
and evidence are resolved.
