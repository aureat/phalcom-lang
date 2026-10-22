# Source-free executable field ownership consultation

## State and required invariant

Continue in `.worktrees/artifact` on `feat/artifact-universe-consolidation`.
The revision anchor is `e4202ce348c89f058539ef6509206b4eed8ddd60` plus
uncommitted work. P2.R4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`;
P3 remains blocked. No source-free execution has passed.

Every executable field access must have a proven P7-owned class layout and
an in-range slot. Preserve strict validation, canonical identity, and existing
VM installation semantics. Do not use display names, the first matching field,
or source fallback to supply missing authority.

## Observed

The exact full-artifact SlotMap test now stops during executable validation:

```text
prototype 43 (Nested { parent: PrototypeId(13), ordinal: 17 }) uses field slot 0 without a P7-owned class layout (resolved class None)
```

The latest diagnostic run selected one test and failed in 100.16 seconds.
Log: `/tmp/artf001-p2r4-source-free-slotmap-r7.log`.
The reusable artifact `/tmp/artf001-p2r4-full-universe.telosc` does not exist:
build validation fails before the test writes it. User compilation, artifact
VM bootstrap, direct execution, and fresh-process denial have not been reached.

## Causal evidence

- `artifact/capture.rs::capture_object`, closure arm: ordinary closure
  constants receive `PrototypeOwnerV1::Nested { parent, ordinal }`.
- `artifact/capture.rs::capture_detached_methods` supplies canonical Callable
  owners for detached methods. Ordinary nested closure capture does not use
  that path.
- `artifact/validate.rs::prototype_class`: Nested follows its parent;
  ModuleInitializer and conformance-owned Callable return no class. Declaration
  and variant Callable owners map to their declaration and dispatch side.
- `compiler/lib/mod.rs` emits closure templates with `lexical_class: None`.
- `vm/dispatch.rs`, `Bytecode::Method`: installation assigns holder,
  access owner, lexical class, and super anchor using the runtime receiver.

Hypothesis: prototype 43 needs installation/activation ownership proof that
parent traversal cannot establish. The observed failure proves the missing
class result; it does not identify a unique authorized installation target.
No corrective attempt has been made for this ownership issue.

## Decision requested

Identify the canonical proof that should establish the class layout for this
nested field-using prototype. The recommended direction is the existing P8 T3
schema-preserving CFG installation-provenance design: prove the Method receiver,
method recipe and activation relationship, then derive field context from that
validated relationship. Confirm whether that work should precede this source-free
gate and define its required field-context handoff. Preserve fail-closed handling
of ambiguous, reused, or conflicting installation sites.

The alternative of tagging capture with additional owner authority would require
an explicit identity/representation decision and cannot be substituted silently.
Inferring a class from a matching name or trusting an unproved lexical anchor is
not an accepted correction.

Repository escalation triggers: canonical ownership/proof and VM control-flow
soundness are uncertain; completing the missing proof expands the supplied
validator-regression continuation. Follow
`docs/workflow/shared-consultation-escalation-protocol.md` before editing this issue.
No consultation subagent was used.

## Completed changes and focused evidence

- Added class-tower regression accepting instance-side Metaclass references and
  Object-class to Class inheritance, while rejecting wrong metaclass side and
  superclass cycles.
- Added fused InvokeConst/InvokeLocal regression for both validator traversals,
  including reachable underflow and missing fallthrough rejection.
- Updated the validator signature contract test for finalized linked authority.
- Validator tests pass 3/3, zero filtered-in omissions; final log:
  `/tmp/artf001-validator-regressions-final.log`.
- Fixed executable semantic projection to preserve every bytecode-addressed
  call-environment source slot, including duplicate equal recipes. Nested
  product references may still reuse an existing entry.
- The new projection regression passes 1/1; log:
  `/tmp/artf001-call-environment-regression.log`.
- Full run r5 first exposed a call-environment index 1 against a table of length 1.
  After the projection correction, r6 reached the field-ownership failure.
- Added a fresh-process execution runner and parent invocation under
  `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`. It asserts empty loaded sources, source-free
  snapshot completeness, zero canonical bootstrap count, artifact-backed
  compilation, VM bootstrap, and SlotMap execution. The gate is wired but unrun
  because direct validation has not passed.

Strict parity, materializer/runtime tests, allocation certification and broad
suites were deliberately deferred. At this incident snapshot, changes remained
uncommitted and unpushed.

## Next action after the ownership decision

Implement the accepted proof and add tiny positive/negative ownership tests.
Then rerun the single full-artifact test with the cache path. Once validation
allows the artifact to be written, reuse its bytes for runtime materialization
work; rebuild only after capture, schema, linked authority, or artifact contents
change. Semantic-pool reconstruction remains an expected later materializer gap;
it has not been exercised or changed in this continuation.

## Consultation resolution and implementation continuation — 2026-10-22

**Consultation verdict: PLAN ARCHITECTURE SOUND / IMPLEMENTATION GAP ONLY.**
The active P8.T3 plan already specifies the required CFG installation proof.
Proceed with that proof before materialization or VM execution. Do not change
`PrototypeOwnerV1`, the executable schema, capture-time owner assignment, or VM
installation behavior. The authoritative field context is the source
declaration plus portable dispatch side; for a class-side method it is the
source class's `Class` layout, not its runtime metaclass holder.

Implementation now includes conservative CFG value provenance for method
recipes, class allocations, receiver values, method installations, closure
activation, and field-opcode receivers. Structural operand checks remain
whole-image; field authority is checked only from proven receiver/layout facts.
The validator regression suite passed 12/12. The full artifact gate progressed
past earlier failures but currently fails on an upvalue receiver. The exact
latest failure is:

```text
Executable("prototype 89 instruction 1 has unknown receiver provenance for field slot 2: receiver provenance is Some(Unknown); owner Nested { parent: PrototypeId(88), ordinal: 0 }, name \"<closure>\", code around access [GetUpvalue { operand0: 0 }, GetField { operand0: 2 }]")
```

The first failing phase is executable validation during full Universe artifact
construction. The intended cache path is
`/tmp/artf001-p2r4-full-universe.telosc`; no successful artifact was created,
so there is no artifact to load or reuse. Artifact-backed compilation, VM
bootstrap, runtime execution, and fresh-process source-denied execution have
not run. Continue by proving captured upvalue receiver provenance through the
validated closure activation/capture relationship; preserve fail-closed
behavior for unknown or conflicting authority.

## Remote P8.T3 upvalue-provenance implementation slice — 2026-10-22

Remote `feat/artifact-universe-consolidation` receives three focused commits:

- `cae2a50b4` — activation provenance now carries separate receiver authority and captured upvalue values; `GetUpvalue` and `GetUpvalueChecked` use captured values, not a method holder or lexical-class guess. `Local` resolves against the actual parent's abstract slot at the creating `Closure` instruction; `Upvalue` forwards the parent's captured value.
- `20cecea6d` — focused regressions for instance/class-side captured receivers, forwarded captures, unknown and mutated cells, and out-of-range field slots.
- `0004cbf24` — `Closure` rejects a template whose nested owner does not identify the executing parent, preventing forged parent-frame capture authority; regression added.

This is a **bounded, unverified implementation slice**. No cargo tests, full-artifact test, CI action, or remote build has been run. The GitHub file integration cannot execute `cargo check`; the implementer requested local tests. The previous prototype 89 failure remains **pending revalidation** and must not be marked fixed solely because the transfer rule was added.

The mutable capture guard conservatively drops exact provenance if the creating frame can write the captured local after closure creation, if the closure creation is cyclic, if scratch-slot rearrangement can invalidate the local, or if any nested closure sharing the cell can execute `SetUpvalue`. A per-prototype join requires agreement across every observed activation. This does **not** change `PrototypeOwnerV1`, executable wire schemas, P7 stable identities, VM installation, or source fallback policy.

Local gate order:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib

RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib artifact::validate::tests:: -- --nocapture

PHALCOM_TEST_FULL_TELOSC_PATH=/tmp/artf001-p2r4-full-universe.telosc \
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
  modules_canonical_artifact::artifact_executable_bootstraps_and_runs_slotmap_without_universe_compilation \
  -- --exact --nocapture --test-threads=1
```

If compilation fails, correct the concrete compiler error before running the 100+-second full-artifact gate. If the full test fails, record the **first actual failure** with prototype, capture descriptor, creation-site parent state, runtime receiver and branch head; do not add speculative P7 authority. Two deferred `Module.metadata` semantic diagnostics and all unexercised source-free runtime/parity gates remain open.

Lifecycle: P2.R4 `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` (existing prior evidence only); P3 `BLOCKED`.

### CFG mutation-reachability hardening (same unverified slice)

Additional remote commits:

- `2630efb1a` — replace textual-index comparisons with reachable CFG successor traversal when searching for post-capture local writes. This rejects a backward branch that mutates a captured local after `Closure` without revisiting the creation instruction. A focused negative regression covers that topology.
- `6cc21742e` — remove an unused parent binding after the CFG refactor.

The full artifact, Rust compile checks and regressions remain **unrun**. The proposed upvalue transfer is not claimed to resolve prototype 89 until the local gate proves its captured producer and mutation constraints permit exact authority. Continue with the exact local gate commands above and report the first compiler error or executable validation failure.

## Local verification and downstream semantic-pool boundary — 2026-10-22

Local verification of the upvalue-provenance slice passed the complete focused
validator selection: **22 passed, 0 failed, 0 ignored**. This includes exact
instance/class-side captured receivers, nested forwarded upvalues, unknown and
mutated captures, backward-branch mutation, conflicting activations, forged
closure parentage, method-installation ownership, generated enum/data receivers,
class-tower checks, and slot-range rejection. P8.T3's focused proof is therefore
verified at the validator-test boundary. The full-artifact discriminator has
not yet been rerun, so prototype 89 is not declared cleared until that gate
observes the new analysis.

Static audit of the next load/runtime stage identified an independent concrete
materializer gap: `artifact/materialize.rs::materialize_callable` reconstructs
portable constants and bytecode into a fresh `Chunk` but leaves
`Chunk::executable_semantics` empty. Portable prototypes already reference an
artifact semantic pool and capture serializes those pools, so silently executing
with empty native side tables is incorrect.

The first corrective downstream slice separates checker-only trait proof state
from runtime executable evidence. `Chunk` now retains the source
`TraitInvocationSpec` for artifact projection and a same-index
`ExecutableTraitInvocationSpec` for VM execution. VM trait dispatch consumes
only the exact selected callable/data-component target, requirement identity,
requirement/antecedent slots, runtime type bindings and call-environment recipe.
Artifact capture requires both source/runtime views to agree exactly before
serializing. The unpublished portable trait row now retains the otherwise-lost
`selection_is_trait_default` bit needed to preserve direct requirement-slot
installation semantics after source-free load. No accepted full V1 executable
artifact exists on this branch, so this corrects the unpublished schema draft
rather than introducing compatibility with already-authorized bytes.

Commits in this downstream slice through this record:

- `e971c24d3` — split runtime trait invocation evidence from checker proof.
- `186a0be7f` — explicitly classify the runtime projection during portable capture.
- `d954cdb72` — make VM trait execution consume runtime-only evidence.
- `abb60222e` / `c79019cc9` — persist and project exact trait-default execution mode.
- `2c3c99953` / `a3346512c` — expose the canonical runtime projection internally and
  fail artifact projection if semantic/runtime same-index views diverge.

This downstream slice is **not compile-verified yet** and does not yet install
`PortableSemanticPoolV1` on materialized chunks. Data-construction runtime
projection and the remaining portable side-table relocation are deliberately
left for the next bounded slice after compilation of this boundary is confirmed.
P2.R4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; P3 remains blocked.
