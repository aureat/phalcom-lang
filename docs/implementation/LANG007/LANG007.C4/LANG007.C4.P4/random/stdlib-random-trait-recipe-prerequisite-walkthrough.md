# Trait method-generic recipe prerequisite walkthrough

Historical prerequisite record. The subsequent random delivery walkthrough
`stdlib-random-completion-walkthrough.md` records the current implementation.

Status: COMPLETE. Completion: IMPLEMENTED. Verification: FOCUSED_TESTED.
Branch: main. Starting HEAD: `800474d8` (stable owner repair: `fbf057ea`).
This continuation is uncommitted.

## Root cause and repair

Application inferred the trait requirement's method binders while the executable
recipe used the selected source witness's binders. The first attempted correction
could not find the selected signature in top-level analysis. The conformance
semantic view already provided that canonical signature. Lookup now consumes it
and records the existing callable-signature dependency.

Publication checks the requirement contract first, retains discharged proofs,
and translates alpha-equivalent method parameter bindings to the source witness.
The compiler's selected-trait lane also failed to carry its existing call-entry
recipe into VM execution. `TraitInvocationSpec` now carries it, metadata pool
remapping visits it, and VM activation materializes both type and conformance
environments before executing the witness. Abstract default slots follow the
selected callable's generic prefix. Existing evidence/witness identities are
preserved; runtime performs no conformance inference.

Production files:

- `phalcom-semantic/src/checker/{call,context}.rs`
- `phalcom-core/src/modules/{semantic_lowering,materialize}.rs`
- `phalcom-core/src/vm/dispatch.rs`

## Verification

The unchanged original random syntax corpus passes. Core trait tests pass 16/16.
New semantic tests cover selected witness binder/proof publication, missing-bound
rejection, and cold/incremental parity. The core regression covers two renamed
binders, reordered witness bounds, exact and forwarded proofs, nested default
dispatch, typed allocation, and forced GC. Core checking and diff hygiene pass.
The incident record owns exact commands and four confirmed semantic-suite
baseline failures.

Workspace/release certification, performance measurement, and the random
library final gates were not run. A provisional random slice exposed separate
canonical Universe constant/default-product gaps and was removed. Parent random
plan completion is not claimed.

## Consultation and inherited constraints

The user renewed repair authority and required an architecturally coherent fix.
Stable source ownership, exact conformance evidence, witness selection, and the
original R0 trait/default/associated-type design remain intact. The completed
prerequisite introduces no random-specific runtime behavior.
