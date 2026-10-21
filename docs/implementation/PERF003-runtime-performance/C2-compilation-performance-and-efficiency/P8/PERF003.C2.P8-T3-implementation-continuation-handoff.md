---
id: PERF003.C2.P8
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
date: 2026-10-21
branch: feat/artifact-universe-consolidation
based_on_revision: 180cee9ebeeb524c7a10f6018e14ede143e4bcf9
implementation_revision: pending-current-implementation-commit
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
| P8.T3 | PARTIAL / UNVERIFIED | Every opcode has an explicit generated control-flow class, and reachable branch/fallthrough bounds are checked. Stack/frame proofs, owner-specific exit validation, and validator behavior tests remain open; no validated authority is minted. |
| P8.T4–T7 | NOT_IMPLEMENTED | Materialization, source-free native floor, artifact-backed VM construction and deferred imports remain ahead. |
| P8.T8–T9 | PARTIAL | Full publication/CLI workflow and acceptance harnesses/evidence remain open. |
| P8.T10 | OPEN | Close P8 only after the required behavior and gate evidence exist. |

At continuation start, `feat/artifact-universe-consolidation` and its origin
tracking branch both pointed to
`180cee9ebeeb524c7a10f6018e14ede143e4bcf9`. The checkpoint is the canonical
shared lifecycle record; this file is a takeover aid, not a second plan or a
P8 completion handoff.

## Implemented slice and findings

- `phalcom-compiler-artifact/src/executable/instruction.rs` defines
  `PortableOperandValueV1` and a generated `visit_operands` method. It derives
  operand values and domains from the opcode declaration, avoiding a second
  hand-maintained opcode-field match in core. It now also requires every
  opcode row to declare `ControlFlowKindV1`; the manifest and generated
  `control_flow_kind()` accessor derive from that same row. Adding an opcode
  without a control-flow class fails macro expansion instead of defaulting to
  fallthrough.
- `phalcom-core/src/artifact/validate.rs` adds typed validation errors, a
  private-field `ValidatedExecutableImage`, and a validation context tied to
  `LoadedUniverseArtifact` and its P7 module/semantic authority. The validation
  entry point checks structural limits, executable ABI versions, the native
  ABI digest, the Debug profile, module/bootstrap relations, native and
  primordial records, class/prototype/closure layouts, operand domains, and
  semantic-pool references.
- The reachable CFG validator now uses generated flow classes instead of
  opcode-name strings. It checks that only branch classes carry exactly one
  relative-jump operand, validates branch targets, enqueues both successors
  for conditional branches, rejects fallthrough to `code.len()`, and treats
  function exits, non-local exits, and explicit exceptional terminators as
  having no normal successor. Dispatch confirms the relative offset is based
  on the IP after the current instruction. The declared terminators include
  `Return`, `ReturnNonLocal`, `RaiseUnsupported`,
  `RaiseNonIterableStarOperand`, and `MatchInvariantFailure`.
- `phalcom-core/src/artifact/mod.rs` registers the validator module.
- `validate_executable_image` currently returns `Result<(), _>`; it does not
  construct `ValidatedExecutableImage`. The type therefore is not yet an
  execution authority and must not be accepted by a VM API.
- The structural artifact validator checks the executable schema, portable
  bytecode ABI, emitter ABI, native runtime ABI, VM operand/frame ABI, and
  configured image limits. Core also recomputes the native ABI digest and
  requires the Debug compile profile. Confirm remaining compatibility fields
  and all required resource budgets before promoting the result to authority.
- Control-flow classes and reachable branch/fallthrough bounds are now
  explicit, but stack safety is not proven. `FunctionState::max_slots` is the
  peak number of live locals; it is not a declared operand-stack maximum.
  Stack analysis must therefore derive the frame entry/local floor from the
  compiler/runtime contract rather than treating `max_slots` as an operand
  limit. Several instructions need edge-specific transfers: for example,
  `JumpIfUnsupported` pops on its taken edge and restores the value on
  fallthrough, while `TryInvokeExact` truncates receiver/arguments on its
  missing-method edge. Do not model these as one net stack delta. Owner-specific
  return validity, stack joins/underflow, and reachable frame-slot safety also
  remain open. The manifest's explicit control-flow class alone is not proof
  of VM execution safety.
- The current P7 context is built from the loaded artifact. Do not rebuild
  canonical semantic identity, linked interfaces, or module topology from
  executable storage or Universe source.

## Next implementation work

1. Continue only the remaining T3 compatibility/resource and cross-section
   checks; reuse `ExecutableImageV1::validate_structure` and the already-loaded
   P7 context. Do not duplicate the structural decoder or rebuild authority.
2. Add edge-sensitive stack transfer rules for the generated opcode classes
   using concrete VM dispatch semantics. First establish the actual frame entry
   height and local lifetime behavior; `max_slots` is a local-slot peak, not a
   stack-depth cap. Reject reachable underflow, inconsistent join heights,
   invalid fallthrough, and frame-slot accesses that are not backed by the
   reachable frame state. Preserve different outgoing stack states for
   conditional edges such as `JumpIfUnsupported` and `TryInvokeExact`.
3. Validate owner-specific return/non-local-return rules and ensure every
   operand/reference remains checked across the whole prototype, including
   unreachable instructions where the structural contract requires it.
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
  warnings for the unintegrated validator and existing semantic imports. This
  is compilation evidence, not a validator behavior proof.
- `git diff --check`: PASS. A control-flow manifest test was added but not run;
  no validator behavior test was run.
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

Start with this handoff and the amended plan's current reconciliation table.
The checkpoint is authoritative for status and gates. Inspect only exact owners
needed for the next stack-proof edit:

- `phalcom-core/src/artifact/validate.rs`, especially
  `validate_executable_image`, `validate_instruction_operand`, and
  `validate_prototype_control_flow`;
- `phalcom-compiler-artifact/src/executable/instruction.rs` for generated flow
  and operand metadata;
- `phalcom-core/src/vm/dispatch.rs` plus the exact frame-entry owner for
  edge-sensitive stack effects and the initial stack state. Do not inspect all
  dispatch arms at once; follow the stack-rule slice being implemented.

Run only the authorized core library check after a coherent change. Broaden
verification only if the owner changes the policy or a later P8 gate is
explicitly authorized.
