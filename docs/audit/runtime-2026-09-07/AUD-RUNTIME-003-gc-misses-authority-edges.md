# AUD-RUNTIME-003 — GC tracing omits retained lexical and layout authority

## Classification

- Severity: High
- Category: Runtime Safety, Memory, Correctness, Architecture
- Confidence: Confirmed missing lexical-class edge; related omitted fields confirmed by source inspection
- Priority: Fix before further runtime identity work
- Runtime impact: A live closure can retain a stale class handle after collection; later authority/layout checks can panic.
- Affected components: ClosureObject, MethodObject, CallFrame, GC tracing

## Executive Finding

A rooted closure's `lexical_class` does not keep that class alive. The diagnostic probe rooted the closure, forced GC, and observed `lexical_owner_survives=false`. This is a missing strong edge, not pointer unsafety: the arena detects stale handles and its public `get` path panics.

## Observed Architecture

The collector enumerates object variants exhaustively, but payload fields are not destructured exhaustively. Adding a handle-bearing field to an existing type does not require its tracing arm to change. The trace file itself acknowledges this class of maintenance risk.

## Evidence

- `phalcom-core/src/heap/closure.rs:38`: `lexical_class: Option<ClassId>` and `foreign_receiver_guard` retain semantic authority.
- `phalcom-core/src/heap/trace.rs`, `Object::Closure` arm: visits module, upvalues and constants; omits lexical class and foreign layout owner.
- `phalcom-core/src/method/object.rs:181`: `access_owner` exists separately from `holder`; `Object::Method` tracing visits holder but omits access owner.
- `phalcom-core/src/frame.rs`: `ForeignReceiverGuard.layout_owner`; `trace_frame` visits closure and context but not this guard.
- `phalcom-core/src/vm/send.rs`, `current_access_class`, `authorize_method_access_as`, `guard_foreign_layout_access`: consumers of the omitted references.
- `phalcom-core/src/heap/mod.rs:309`: `Heap::get` panics on dangling handles.
- [Probe](evidence/runtime_audit_probe.rs) and [output](evidence/probe-output.txt).

## Runtime / Compilation Path

Method installation/materialized closure retains authority handle → closure survives through a root → collector omits authority edge → class swept when no other root retains it → access/hierarchy/layout operation consumes stale handle.

## Invariant

Every strong handle consumed by a live runtime object or frame remains live through collection, unless an explicit weak-handle contract checks and tolerates expiry.

## Current Behavior

Normal registered classes often survive independently through `VM.classes`, masking the omission. The probe deliberately removes that alternative retention route. It proves the missing edge, not a full source-level access-control crash. REPL replacement, escaping blocks and transplanted methods are the next source-level cases to establish.

## Failure Scenario

A retained closure has a lexical authority owner that is no longer the module's current class binding. If no receiver, constant or other object retains that owner, GC can reclaim it. The closure can subsequently use its stale authority in a protected hierarchy walk or error rendering.

## Root Cause

GC completeness is mechanically enforced for enum variants and top-level VM fields, but not for fields within frame/object payloads. Authority and layout handles were added outside the original trace inventory.

## Impact

### Runtime safety

Premature reclamation of logically live metadata; potential Rust panic during later lookup. No raw-pointer use-after-free or undefined behavior was demonstrated.

### Architecture

Retained execution identity and GC retention must evolve together. Fixing super anchoring by adding another untraced handle would repeat this defect.

## Recommended Direction

Trace lexical owner, distinct method access owner, and foreign layout owner wherever stored. Use exhaustive payload destructuring or a per-payload tracing implementation so adding a field forces an explicit edge decision. Separate weak caches from strong authority edges rather than indiscriminately tracing every cache.

## Alternative Solutions

Permanently rooting every historical class masks omissions and leaks obsolete classes. Adding `try_get` everywhere prevents a panic but loses required authority instead of preserving it.

## Implementation Outline

Repair the proven edges, enumerate related frame/closure/method handle fields, and add one-edge-only GC tests. Review `RuntimeTypingRegistry` and associated-target caches separately: these also contain handles but their intended strong/weak lifetime contract was not established in this checkpoint.

## Required Tests

Retain each authority owner solely through the field under test; force collection; verify survival and successful use. Drop the holder and verify eventual collection. Repeat for parked frames, nested closures and method transplantation. Add a real REPL replacement/access-control scenario under GC stress.

## Verification Criteria

Every live authority reference is traceable; its owner survives independently of name registries. Weak entries either invalidate or check expiry before consumption.

## Related Findings

[002 — lexical super identity](AUD-RUNTIME-002-super-dispatch-loses-lexical-identity.md).

## Open Questions

Which ordinary source scenario first exposes an omitted authority edge without another retaining reference? Which typing/associated caches intentionally use weak references? Neither has been conclusively established yet.
