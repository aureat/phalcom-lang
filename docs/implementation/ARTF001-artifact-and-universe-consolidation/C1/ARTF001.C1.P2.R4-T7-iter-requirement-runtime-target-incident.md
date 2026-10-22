# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `ARTF001`
- Checkpoint: `ARTF001.C1`
- Plan/gate: `ARTF001.C1.P2.R4`, T7 iterator runtime
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Worktree: `.worktrees/artifact`
- Branch: `feat/artifact-universe-consolidation`
- Base revision: `4f0861428`

## 2. Escalation trigger

Trigger 8 applies: the iterator gate reaches a semantic-to-VM conformance
contract failure. Trigger 11 applies because the missing target could come
from semantic selection, compiler projection, or VM environment propagation.

## 3. Required invariant

The VM must invoke only a target authorized by canonical semantic evidence.
The compiler must preserve that selection in the runtime environment. Do not
rediscover a conformance target from the receiver or fill the missing slot by
name.

## 4. Expected architecture

The checked `Iterable.iter` default call carries the selected conformance and
its requirement targets into a runtime conformance environment. Its body then
invokes the abstract `Iterable.iterator` requirement through the active slot.

## 5. Observed behavior

The focused CLI compiles the fixture and enters `Iterable.iter`. Evaluation of
`self.iterator` fails because the active environment has no target for the
`Iterable.iterator` requirement. This is a VM internal error, not a source
diagnostic.

The failure-only trace now distinguishes the boundary: semantic selection
contains the concrete `Array` conformance target, and the caller environment
contains that target at slot 29. The compiled `Iterable.iter` body reads slot
55 from the same environment, whose length is 41. This is a slot-layout
mismatch between caller projection and the default body's abstract requirement
projection.

## 6. Minimal reproduction

```sh
RUST_MIN_STACK=16777216 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo +nightly-2026-07-10 --config 'build.rustc-wrapper=""' \
  run -p phalcom-core --bin phalcom -- \
  phalcom-core/tests/fixtures/language/iterator/iterator_skip_take_takewhile.ph
```

Observed error:

```text
iterator_laws.ph:44 in iter
no active conformance target for requirement Iterable.iterator
```

The process exited with status 70 after compiling and running the fixture.

## 7. Relevant code path

```text
lowered trait invocation
  -> VM::invoke_trait_selected
  -> VM::materialize_trait_environment
  -> default Iterable.iter frame
  -> abstract Iterable.iterator invocation
  -> VM::invoke_trait_requirement reads the active requirement slot
```

Key locations:

- `phalcom-core/src/modules/semantic_lowering.rs` projects
  `TraitDispatchSite::Evidenced`.
- `phalcom-core/src/vm/dispatch.rs` materializes the trait environment and
  invokes abstract requirements.

## 8. Scoped implementation diff

The active work already contains the T7 associated-projection identity fix and
the narrowly gated call-completion proof for a fully materialized equality
proved by lexical evidence. The new generic-chain semantic regression passes.
No runtime conformance-slot behavior has been changed for this incident.

## 9. Evidence

- The exact CLI moved beyond the earlier constructor inference diagnostic and
  reached VM execution.
- The reported requirement is `Iterable.iterator`, at the body expression
  `self.iterator` in `Iterable.iter`.
- `invoke_trait_requirement` reads its slot from the current frame's
  conformance environment and errors when the slot is absent.
- `materialize_trait_environment` allocates slots for a trait default and
  populates requirement slots only when the selected dispatch product contains
  a matching `requirement_targets` entry.
- The exact selected target is `Iterable.iterator` to the `Array` conformance
  implementation's `iterator` getter (`ImplLocalId(3)`).
- Caller-side lowering maps the exact Getter/Instance requirement to slot 29.
- VM materialization creates environment 1 with 41 slots; slot 29 is populated.
- The abstract invocation inside `Iterable.iter` uses raw requirement slot 14
  plus a generic-evidence prefix of 41, producing slot 55. The active frame
  inherits environment 1 and requests slot 55, outside its 41-slot vector.
- Temporary trace code has been removed. The local trace was captured at
  `/tmp/phalcom-iter-target-trace.log`.

## 10. Implementer's causal hypothesis

The selected semantic evidence is complete for this call. The mismatch is in
compiler/runtime slot layout: the selected invocation projects the target at
29 while the compiled default body requests it at 55. The same runtime
environment reaches the failing frame, so the trace does not point to
environment-ID propagation loss.

## 11. Alternative hypotheses

- The call-entry recipe and the compiled default-body layout may require
  explicit canonical slot mapping when their evidence layouts diverge.

## 12. Attempts already made

No runtime behavior correction has been attempted. A single failure-only
trace compared semantic selection, compiler projection, VM materialization,
and the failing frame read. The source-level call completion change was
independently gated by semantic proof and regressions.

## 13. Why local authority is insufficient

Choosing among semantic proof publication, compiler slot projection, and VM
environment propagation changes the authority boundary for abstract trait
requirements. A fallback target lookup would create runtime semantic
rediscovery and violate the required invariant.

## 14. Advisory ruling and focused acceptance

The adviser requested comparing the exact requirement through semantic
selection, compiler slot projection, VM environment creation, and the failing
frame read. The ruling assigns ownership to compiler lowering/runtime layout
projection. Keep slot 29 as the lexical `Self |= Iterable` evidence entry and
slot 55 as the direct abstract requirement entry; do not replace one with the
other. Both must be populated from the same exact selected semantic target
through the canonical callee-body generic-evidence layout and trait-requirement
layout. Keep the VM fail-closed and do not add receiver/name lookup.

The focused discriminator must prove the lowered requirement uses slot 55,
materialization places the exact `Array.iterator` target at both slots 29 and
55, a generic forwarded call still uses slot 29, and an absent target still
fails closed. Then rerun the exact iterator CLI gate. This contract amendment
is recorded in the active P2.R4 plan and checkpoint.
