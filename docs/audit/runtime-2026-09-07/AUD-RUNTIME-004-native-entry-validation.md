# AUD-RUNTIME-004 — Native entry validates too late or relies on callers

## Classification

- Severity: High for the native extension boundary; source reachability of unbounded native-only recursion remains unverified
- Category: Runtime Safety, VM, Compiler-Runtime Contract
- Confidence: Confirmed through public VM APIs and a bounded test primitive
- Priority: Fix before expanding native integrations
- Runtime impact: Wrong-arity host sends panic in shipping primitives; native-only recursive sends bypass the advertised re-entry ceiling.
- Affected components: VM::send_dynamic, legacy primitive activation, native re-entry accounting

## Executive Finding

`send_dynamic(Value::int(1), selector("+(_)"), &[])` reaches the shipping `number_add` primitive and panics at `args[0]`. A deliberately bounded native primitive recursively sending itself 40 times succeeds despite `MAX_NATIVE_REENTRY = 32`. Both arise because the activation boundary is entered before all of its preconditions are established.

## Observed Architecture

Ordinary source sends normally encode arity into the selector and reject more than 255 arguments. Public synchronous native sends independently accept a selector and an argument slice. Exact lookup does not reconcile them. The native depth counter wraps `run_until`, after dispatch has already executed a selected native primitive.

## Evidence

- `phalcom-core/src/vm/send.rs:1038`, `send_dynamic`: pushes receiver/arguments; dispatches exact method; only then calls `check_native_reentry` and increments the counter.
- `phalcom-core/src/vm/send.rs:247`, `call_method_legacy`: invokes a legacy primitive using the caller-provided argument count without matching its signature.
- `phalcom-core/src/primitive/number.rs:269`: shipping numeric addition indexes `args[0]`.
- `phalcom-core/src/vm/api.rs:264`: ceiling check.
- `phalcom-core/src/vm/mod.rs:71`: `MAX_NATIVE_REENTRY = 32`.
- `invoke_method_object` performs an argument check, demonstrating that validation differs between public entry paths.
- [Probe](evidence/runtime_audit_probe.rs), [output](evidence/probe-output.txt): `SHIPPING_NATIVE wrong_arity_panics=true`; `NATIVE recursion_40=Ok(40)`.
- Existing `tests/core/execution/depth_limits.rs` tests language recursion/native-to-bytecode recursion. Those were read, not executed in this checkpoint.

## Runtime / Compilation Path

Host/native send → independent selector and argument slice → exact lookup → legacy primitive → indexing or recursive send → only after the primitive returns does the outer send check the depth limit.

## Invariant

Before calling native code, arguments match the selected method's accepted shape and every recursive Rust activation is charged against an applicable bound.

## Current Behavior

Compiler-generated exact calls establish ordinary arity by construction. The public native boundary does not. Native-to-bytecode re-entry is bounded, but native-to-native recursion can occur entirely before counter increments. The probe stopped at 40; no actual stack overflow was induced.

## Failure Scenario

1. An embedding or new primitive makes a selector/argument mismatch: a Rust panic replaces a controlled `Arity` error.
2. Two native primitives synchronously send to each other without a bytecode activation: the present counter does not bound that recursion. Continued recursion can exhaust the Rust stack; that consequence is inferred from the demonstrated bypass.

## Root Cause

Validation and depth accounting belong to selected helper paths, rather than the complete native activation lifecycle. The current counter bounds recursive interpreter entry, not all recursive native work.

## Impact

### Runtime safety

Host integration mistakes can terminate or unwind the host unexpectedly. Infinite native recursion is not contained by the advertised guard.

### Architecture

The legacy and shape-aware ABIs have different responsibilities. Their shared entry contract should be explicit before more native gateways are added.

## Recommended Direction

Validate host/native call shape before mutating the stack. Establish a native activation budget before invoking code that can recursively send; keep fiber-switch restrictions and recursive interpreter accounting semantically distinct where needed. Restore counters and authority state on every ordinary error return.

## Alternative Solutions

Defensive indexing in every primitive treats symptoms and leaves recursion unbounded. Moving only the existing check upward without charging the full activation still does not bound native-only recursion. Blanket `catch_unwind` is not a substitute for valid entry state.

## Implementation Outline

Define one admission contract for public synchronous calls, distinguish trusted compiler sends if avoiding redundant hot-path validation is warranted, and scope native activation state around the selected call. Preserve flat `CallOutcome::EnteredFrame` gateways; do not reintroduce recursive interpreter entry into them.

## Required Tests

Missing/excess arguments to real primitives; selector/label mismatch; native-only mutual recursion; native-to-bytecode recursion; exact ceiling neighbors; cleanup after controlled errors; legal fiber transitions and forbidden yield across synchronous native frames.

## Verification Criteria

Wrong host arity returns a controlled error before primitive entry. Bounded native recursion is rejected at the configured limit. Correct source sends and flat closure gateways retain their behavior.

## Related Findings

[005 — execution API cleanup and bytecode trust](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md).

## Open Questions

Is there an existing ordinary-source route to an unbounded native-only cycle? How should native activation limits differ from the existing interpreter re-entry/fiber restriction counter? These need targeted follow-up.
