# IMPLEMENTATION INCIDENT: unsupported arithmetic acceptance needs an oracle decision

## 1. Identity

- Program: STDL001
- Checkpoint: STDL001.C5
- Plan: STDL001.C5.P1
- Task/Gate: T1, unsupported-component/operation coverage
- Repository: `.worktrees/complex-math`
- Branch: `feat/complex-math`
- Starting/current revision: `ad1db231`, uncommitted implementation

## 2. Escalation trigger

The new negative regression assumes compile-time rejection for unavailable
operations. Live checker behavior uses unresolved message-send analysis instead.
The test oracle may be too strong; enforcing it would change general checker
acceptance beyond D4/D5. Mandatory triggers: expected semantic test oracle,
plan/live-contract conflict, and materially expanded semantic scope.

## 3. Required invariant

Complex arithmetic is available only with canonical evidence of the operation's
component capabilities. Int division and Complex numerical ordering are absent.
No dynamic component arithmetic fallback, type-name exception, or runtime probing
may grant conditional members without proof.

## 4. Expected architecture

The semantic layer proves conditional applicability; the compiler consumes it.
The plan says unsupported operations are unavailable but does not explicitly
settle whether every missing operation must be a static error. The newly added
test interpreted that wording as compile-time rejection.

## 5. Observed behavior

All three examples compile and have zero conditional invocation products:

```phalcom
Complex(true, false) + Complex(false, true)
Complex(1, 2) / Complex(3, 4)
Complex(1, 2).compare(Complex(3, 4))
```

They are not selected conditional calls. Missing dispatch is analyzed as
`Unknown(DynamicMessageSend)` with no callable identity and normally Ready status.
This is distinct from formally publishing a `Dynamic` type or a proven capability.
The diagnostic run compiled these examples; it did not execute them.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex::complex_component_capabilities_reject_unsupported_arithmetic -- --exact --nocapture
```

One selected test fails, listing all three accepted examples and zero conditional
call edges for each. Expected by this new test: semantic compile-time rejection.

## 7. Relevant code path

```text
checker/expression.rs::apply_binary_operation_from_typed
  -> resolve_dispatch_target -> ResolvedDispatchResult::Missing
  -> checker/call.rs::analyze_unresolved_application(DispatchMissing)
  -> Unknown(DynamicMessageSend), callable=None, normally Ready
  -> no conditional selected-call lowering product
```

`docs/specs/objects/impl.md` requires proven applicability before a conditional
member enters the available surface. `docs/specs/callables/dispatch.md` separately
defines ordinary terminal misses and `doesNotUnderstand`. Neither establishes
that the word unavailable universally means compile-time rejection.

## 8. Scoped implementation diff

D5 adds complete runtime type comparison, exact data descriptor representative
interning, and conditional call recipe metadata-pool relocation. Its focused
tests pass. The only new availability change is a failing negative regression;
no checker acceptance change has been attempted.

## 9. Evidence

- The exact negative test compiles all three examples successfully.
- Each entry module has zero conditional call edges.
- `analyze_unresolved_application` explicitly uses Unknown knowledge and no
  callable for DispatchMissing, without emitting a missing-dispatch error.
- Complex declares no Int division or compare method.
- D5's three Complex positive tests pass; general data identity and conditional
  result/capture exactness also pass.

## 10. Implementer's causal hypothesis

The test conflates unavailable proven operations with general gradual message-send
acceptance. Conditional applicability correctly withholds the selected capability,
while the existing checker permits an unresolved ordinary send.

## 11. Alternative hypotheses

The intended numeric contract might require a new general static diagnostic for
proven-unavailable operations on closed known receivers. That would be an explicit
checker policy change and must preserve ordinary Dynamic/unknown sends and
intentional message-miss handlers. It cannot be introduced as a Complex exception.

## 12. Attempts already made

No corrective implementation attempt. The negative regression was extended to
report all accepted cases without dumping complete compiled programs. It remains
failing pending a decision; no assertion has been weakened.

## 13. Why local authority is insufficient

Changing the negative oracle or broad checker acceptance changes the plan's
interpreted acceptance contract. User approval is required for significant
semantic/test-oracle choices, and the escalation protocol prohibits self-waiver.

## 14. Decision requested

Recommended D6 clarification: preserve existing gradual message-send acceptance.
Prove unsupported operations have no selected canonical capability and fail at
runtime through the ordinary unavailable-operation path. Amend the negative
oracle accordingly, with explicit compiler/semantic and runtime coverage.

If compile-time rejection is required instead, authorize a general semantic-layer
policy correction and first specify its boundary with ordinary message misses.
No Complex-specific checker/runtime workaround is recommended.

## 15. Adopted decision

The user agreed to the recommended D6 clarification. Negative coverage now
requires absent canonical capability and ordinary runtime failure. The existing
gradual missing-message policy remains the applicable contract. The active plan
and checkpoint record this amendment.

Focused regression passes: one test, three missing operations. Operators use the
existing bilateral UnsupportedOperation terminal; compare uses doesNotUnderstand.
