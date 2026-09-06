# AUD-RUNTIME-002 — Super dispatch follows a replacement class instead of its lexical owner

## Classification

- Severity: High
- Category: Correctness, Dispatch, Architecture, Future Compatibility
- Confidence: Confirmed
- Priority: Fix immediately
- Runtime impact: Existing instances change superclass behavior after another class takes their former name.
- Affected components: Super-send lowering, class registry, VM superclass lookup

## Executive Finding

An old `B` instance returns `1` from `super.value`; after a REPL redefinition of `B` with a different parent, that same instance returns `2`. Its actual class identity did not change. The super-send operand retains a class name, and execution resolves that name through the current registry.

## Observed Architecture

Ordinary instances retain a stable generational class handle. REPL redefinition creates a new class and replaces the name binding. Ordinary method lookup starts from the receiver's old handle. Super lookup instead re-resolves the lexical class's name in the current module.

## Evidence

- `phalcom-core/src/compiler/lib/scope.rs:61`, `compile_super_send`: emits defining class name; static sends encode `<Name>.class`.
- `phalcom-core/src/vm/dispatch.rs:1469`, `SuperSend`: resolves `ClassKey { module, name }`, then reads that class's superclass.
- Dynamic-pack super path in `vm/dispatch.rs` near line 2350 repeats name-based anchoring.
- `phalcom-core/src/compiler/lib/class_decl.rs:636`: REPL class definitions explicitly allocate fresh identities; old instances are not migrated.
- `phalcom-core/src/vm/api.rs:57`, `create_class`: installs new class/metaclass handles into the registry.
- [Executed probe](evidence/runtime_audit_probe.rs), [output](evidence/probe-output.txt): `SUPER before=Ok(1)` and `SUPER old_receiver_after_rebind=Ok(2)`.

## Runtime / Compilation Path

Old instance → old class's getter → `SuperSend(..., #B)` → current module registry's new `B` → new parent `C` → `C.value` runs against the old receiver.

## Invariant

Super lookup begins above the defining behavior of the executing code. Rebinding a name must not retarget code retained by an older class identity.

## Current Behavior

Normal lookup preserves old identity; super lookup switches to the replacement hierarchy. This can also select methods with incompatible assumptions about the old receiver's fields.

## Failure Scenario

First REPL cell:

```phalcom
class A {
  @constructor
  new() {}
  value { 1 }
}
class B is A {
  value { super.value }
}
let b = B.new()
```

Second cell:

```phalcom
class C { value { 2 } }
class B is C { value { super.value } }
```

Send `value` to the retained `b` before and after the second cell. Actual results are 1 then 2. Expected: both 1.

## Root Cause

A presentation/binding key substitutes for execution identity. Module qualification prevents cross-module name collisions but cannot distinguish successive declarations under one binding.

## Impact

### Correctness

Incorrect inherited behavior after supported REPL redefinition. The getter case is reproduced; class-side, constructor-initializer, nested-block and dynamic-pack super variants need companion tests.

### Future extensibility

The same choice is unsuitable for reload, code retained after replacement, and multiple reified behaviors sharing a display name.

## Recommended Direction

Bind an explicit lexical behavior handle when the method is installed, inherit it into blocks, and start super lookup above that behavior. Keep lexical access authority and instance/metaclass dispatch anchor distinct: current `lexical_class` carries access privileges and cannot automatically serve both purposes. Trace any added handle through GC.

## Alternative Solutions

- Starting from the receiver's class is wrong for inherited methods: lookup must start above the method's defining behavior.
- Disabling REPL class replacement avoids the symptom by removing an existing capability.
- Versioned name lookups are possible but more complex than retaining the already-existing identity.

## Implementation Outline

Establish the lexical behavior at installation, preserve it through closure materialization and method transplantation, and use it in exact and packed super paths. Clarify whether transplanted methods retain their source super anchor; do not conflate that choice with private/protected access ownership.

## Required Tests

The two-cell reproducer; static super; inherited constructors; nested closures containing super; rest and dynamic-pack super; old and new receivers coexisting; GC after replacing the original class name.

## Verification Criteria

Old receivers retain old super behavior, new receivers use new behavior, and no super path consults a mutable name binding to recover lexical identity.

## Related Findings

[003 — GC retention of authority handles](AUD-RUNTIME-003-gc-misses-authority-edges.md).

## Open Questions

Exact intended super anchor under reflective method transplantation must be resolved against its effective specification before implementation.
