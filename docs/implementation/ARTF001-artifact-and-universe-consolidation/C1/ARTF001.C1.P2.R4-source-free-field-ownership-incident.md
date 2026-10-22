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
suites were deliberately deferred. All changes remain uncommitted and unpushed.

## Next action after the ownership decision

Implement the accepted proof and add tiny positive/negative ownership tests.
Then rerun the single full-artifact test with the cache path. Once validation
allows the artifact to be written, reuse its bytes for runtime materialization
work; rebuild only after capture, schema, linked authority, or artifact contents
change. Semantic-pool reconstruction remains an expected later materializer gap;
it has not been exercised or changed in this continuation.
