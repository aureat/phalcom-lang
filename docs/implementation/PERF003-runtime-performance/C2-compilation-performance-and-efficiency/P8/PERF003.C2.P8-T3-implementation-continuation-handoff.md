---
id: PERF003.C2.P8
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
date: 2026-10-21
branch: feat/artifact-universe-consolidation
based_on_revision: 18b8aa7a4977e4fadb8c07336919c3636df852fc
---

# PERF003.C2.P8 T3 implementation continuation

## Current state

Continue P8 in place. The owner-supplied closure plan resolved the plan
structure: amend P8; do not create P9. P8 remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

| Task | State | Boundary |
|---|---|---|
| P8.T1 | IMPLEMENTED / FOCUSED_TESTED | Portable executable DTOs, bounded structural decoding, opcode manifest and persistence census are complete. Runtime safety belongs to T3. |
| P8.T2 | IMPLEMENTED / FOCUSED_TESTED | The existing compiler/emitter produces deterministic portable staging data. It is not runtime authority. |
| P8.T3 | PARTIAL / FOCUSED_TESTED | Core validation code exists and compiles. It does not yet mint `ValidatedExecutableImage`; opcode/control-flow/frame proofs and behavior tests remain open. |
| P8.T4–T7 | NOT_IMPLEMENTED | Materialization, source-free native floor, artifact-backed VM construction and deferred imports remain ahead. |
| P8.T8–T9 | PARTIAL | Full publication/CLI workflow and acceptance harnesses/evidence remain open. |
| P8.T10 | OPEN | Close P8 only after the required behavior and gate evidence exist. |

At continuation start, `feat/artifact-universe-consolidation` and its origin
tracking branch both pointed to `18b8aa7a4977e4fadb8c07336919c3636df852fc`.
The implementation work was based on that revision. The checkpoint is the
canonical shared lifecycle record; this file is a takeover aid, not a second
plan or a P8 completion handoff.

## Implemented slice and findings

- `phalcom-compiler-artifact/src/executable/instruction.rs` defines
  `PortableOperandValueV1` and a generated `visit_operands` method. It derives
  operand values and domains from the opcode declaration, avoiding a second
  hand-maintained opcode-field match in core.
- `phalcom-core/src/artifact/validate.rs` adds typed validation errors, a
  private-field `ValidatedExecutableImage`, and a validation context tied to
  `LoadedUniverseArtifact` and its P7 module/semantic authority. The validation
  entry point checks structural limits, executable ABI versions, the native
  ABI digest, the Debug profile, module/bootstrap relations, native and
  primordial records, class/prototype/closure layouts, operand domains, and
  semantic-pool references.
- `phalcom-core/src/artifact/mod.rs` registers the validator module.
- `validate_executable_image` currently returns `Result<(), _>`; it does not
  construct `ValidatedExecutableImage`. The type therefore is not yet an
  execution authority and must not be accepted by a VM API.
- The structural artifact validator checks the executable schema, portable
  bytecode ABI, emitter ABI, native runtime ABI, VM operand/frame ABI, and
  configured image limits. Core also recomputes the native ABI digest and
  requires the Debug compile profile. Confirm remaining compatibility fields
  and all required resource budgets before promoting the result to authority.
- Reachable control-flow validation rejects invalid branch targets and
  fall-through beyond the code array. Its opcode-name match still uses a
  catch-all sequential-flow case. There is no complete opcode control-flow
  classification, stack-effect analysis, reachable frame-slot proof, or
  proof that reachable paths end only at valid exits or intentional loops.
  The manifest describes operands; it does not by itself prove VM execution
  safety.
- The current P7 context is built from the loaded artifact. Do not rebuild
  canonical semantic identity, linked interfaces, or module topology from
  executable storage or Universe source.

## Next implementation work

1. Complete T3's fail-closed compatibility and resource closure. Reuse
   `ExecutableImageV1::validate_structure` rather than adding a duplicate
   structural decoder; identify any remaining compatibility or cumulative
   graph-budget obligations against the active P8 contract.
2. Close module/bootstrap, linked-read, native, primordial, class, constant,
   closure, and all semantic-pool references against the loaded P7 authority.
   Missing authority is an error, never a source reconstruction request.
3. Replace the control-flow catch-all with an exhaustive rule for every row in
   `OPCODE_MANIFEST_V1`. Validate branch destinations, terminators, fallthrough,
   stack effects, and frame-slot safety on reachable paths. The current VM
   dispatch indexes the instruction at the frame IP, so reaching `code.len()`
   is invalid.
4. Only after all checks pass, have the core validator construct and return the
   private `ValidatedExecutableImage`. Keep validation VM-free; do not pass raw
   `ExecutableImageV1` to runtime APIs.
5. Once T3 is complete, start T4 relocation. Preserve existing VM rooting,
   ordering, failure atomicity, and per-VM ownership. T4 relocation may be
   developed before T5's final native-floor integration, but do not execute
   initializers before the validated image and native prerequisites are ready.

Do not mark T3 complete or start runtime materialization based only on the
current compiler check. No validator behavior tests were run in this slice.

## Invariants to inherit

- The existing compiler/emitter remains the only executable producer. Do not
  add an artifact compiler or second semantic checker.
- `PORTABLE_EXECUTABLE` is a claim, not proof. Staged executable data remains
  non-authoritative; the consuming loader must independently validate it.
- Validation must finish before VM construction, native installation, runtime
  allocations, or initializer execution.
- P7 remains the owner of canonical semantic identity, module identity,
  topology, and linked interfaces. Native function implementations remain in
  the current process's native registry; the artifact stores stable keys only.
- No explicit artifact path may fall back to Universe parsing, semantic
  analysis, lowering, source providers, or source-to-bytecode compilation.
- Each VM owns fresh objects, symbols, closures, metadata IDs, caches,
  initializer state, and relocation maps. Share only immutable artifact data.
- Use existing GC roots and preserve observable module/instruction/field/
  parameter/initializer order. Failure must not publish partial VM or deferred
  module state.
- The checkpoint and amended P8 plan are authoritative. Do not re-open the
  settled choice to keep this work in P8, and do not create a successor plan.

## Verification and open gates

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib`: PASS with
  warnings on the implementation worktree. The unintegrated validator emits
  `dead_code` warnings. This is compilation evidence, not a validator behavior
  proof.
- `git diff --check`: PASS. No tests were added or run for the new validator.
- P8.G2 (malformed/incompatible payload rejected before initialization): NOT
  RUN. P8.G3 (source/artifact parity), G4 (fresh-process source-denied eager
  and deferred execution), and G5 (two-VM independence/rooting/sharing): NOT
  RUN. P8.G6 remains PARTIAL: prior two-process payload determinism passed;
  retry and startup timing remain unrun. P8.G7 is DEFERRED.
- For this continuation, the only authorized Cargo command is
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib`. Do not run
  tests, runtime probes, workspace suites, Clippy, benchmarks, or release gates
  without new user authorization.

## Minimal takeover reads

Start with this handoff, the P8 section of `../CHECKPOINT.md`, and the amended
plan's current reconciliation table. Then inspect only the exact owners needed
for the next T3 edit:

- `phalcom-core/src/artifact/validate.rs`, especially
  `validate_executable_image`, `validate_instruction_operand`, and
  `validate_prototype_control_flow`;
- `phalcom-compiler-artifact/src/executable/instruction.rs` for the generated
  opcode manifest and operand visitor;
- `phalcom-compiler-artifact/src/executable/image.rs` for structural limits and
  compatibility checks;
- `phalcom-core/src/vm/dispatch.rs` for instruction execution/termination
  semantics.

Run only the authorized core library check after a coherent change. Broaden
verification only if the owner changes the policy or a later P8 gate is
explicitly authorized.
