# Dynamic Dispatch, Reflection, and Open-World Proof

## The central problem

Phalcom is class-based and message-oriented. A static proof about a send must respect the actual dynamic dispatch semantics, not replace them with a convenient statically selected function call. Optional typing may constrain receiver values, but typing must not silently become a new overload/dispatch rule unless Phalcom explicitly adopts that semantics.

A send has proof-relevant stages:

```text
1. evaluate receiver
2. evaluate arguments in language order
3. determine selector identity
4. determine lookup start / access context
5. perform dynamic lookup
6. invoke selected implementation
7. observe normal or abrupt outcome
```

Each stage may matter. Receiver/argument evaluation can mutate state. Lookup behavior may depend on class/method-table revision. Invocation may mutate, throw, perform reflection, yield, or trigger non-local return.

## Static target sets

A semantic analysis may compute a conservative target set:

```text
Targets(send, facts) ⊇ all runtime implementations reachable under facts
```

For proof, over-approximation is acceptable; under-approximation is unsound. If the set is `{m1, m2}`, a call property must be guaranteed by both targets or by a common protocol/class contract.

Suppose:

```text
m1 ensures Q1
m2 ensures Q2
```

The safe postcondition after dynamic dispatch is generally something implied by both outcomes, conceptually:

```text
Qsafe = join_guarantees(Q1, Q2)
```

In logic this may be encoded with guarded target predicates:

```text
(target = m1 => Q1) ∧ (target = m2 => Q2)
```

or, preferably for modular verification, a contract guaranteed by the dispatch surface:

```text
ProtocolContract(selector).ensures = Q
```

which every conforming implementation must satisfy.

## Behavioral subtyping and override contracts

If callers verify against a base/protocol contract, overriding implementations must be substitutable for that contract. A traditional rule is:

```text
base precondition  => override precondition

override postcondition => base postcondition
```

This means an override may accept at least the states the base promised to accept and must guarantee at least what the base promised to guarantee. Effects/throws/control outcomes also need substitutability constraints.

Do not copy Liskov rules mechanically without checking Phalcom's actual inheritance/protocol semantics, visibility, metaclass behavior, method-family rules, and reflective mutation. But some explicit compatibility rule is needed if dynamic dispatch is to support modular proof.

## Open-world mutation

If methods can be installed/replaced reflectively, a source-derived target set can become stale without any source edit to the caller. A proof therefore needs a closure assumption such as:

```text
DispatchRevision(ClassId, SelectorId) = r42
```

or a stronger package/world revision. Proof dependencies must include whatever state controls lookup.

Possible policies:

- **Open world:** dynamic target may change; rely only on protocol contracts valid for all future installations, otherwise `Unknown(OpenWorldDispatch)`.
- **Closed compilation world:** package/build seals relevant classes/method tables for the artifact; prove against the closed target set and invalidate on mutation/reload.
- **Runtime-guarded specialization:** prove under a dispatch-version guard; optimizer/runtime checks the guard before using the proof-dependent optimization.

The language/runtime must define which policy is real. The prover must not invent sealing.

## Reflection

Reflection creates two distinct questions:

1. Is reflective metadata observable but immutable?
2. Can reflection mutate dispatch-relevant state?

Reading method metadata can still invoke user code depending on APIs, but mutation is the major proof hazard. If reflection can add/remove methods, alter class hierarchy, or change annotations/contracts, proof facts about dispatch and invariants require revision dependencies or runtime guards.

A proof cannot use “the source index currently contains one method” as a substitute for a language guarantee that only that method can execute.

## Metaclasses and class-side sends

Class objects are runtime receivers too. Proof logic should model class-side dispatch using the same selector/lookup principles, not a separate ad hoc static-function namespace. If metaclass state can mutate, proof dependencies must cover it.

## `super`

A `super` send usually has a different lookup start while retaining the current receiver. Proof summaries must reflect that distinction:

```text
receiver = self
lookup_start = lexical/current-class superclass context
```

Do not model `super` as sending to a different receiver object. Its target set may be more constrained than an ordinary dynamic send but still depends on inheritance/method-table semantics.

## Message-not-understood / lookup failure

If a send may lack a method, that is an abrupt outcome unless semantic/type facts prove the selector exists. A proof that assumes a normal return must either prove dispatch success or include the lookup-failure path in the effect/control summary.

## Type information and dispatch

A language type may establish a guaranteed member surface. This is useful:

```text
Γ ⊢ r : P
P guarantees selector s with contract C
----------------------------------------
prove send r.s(...) using C
```

This does not mean `P` statically chooses a method implementation. It establishes a contract all runtime targets must honor.

An IDE `ValueShape` saying “probably instances of Foo” is weaker. It may improve completion but must not become proof evidence unless its analysis has a documented soundness theorem and provenance.

## Example: unsound guessed target

Suppose analysis sees:

```phalcom
x = Account.new()
x.withdraw(10)
```

and chooses `Account#withdraw` for proof. If user code can later reflectively replace that method or `x` can flow to a subclass override, proving with the single body is unsound. Safe alternatives are:

- prove `x` has exact/sealed runtime class `Account` under a stable world revision;
- use a class/protocol contract that all valid overrides satisfy;
- retain the runtime check / return Unknown.

## Effect summaries across dispatch

For target set `T`, the safe may-effect summary is the union:

```text
MayWrite(send) = ⋃ MayWrite(t) for t ∈ T
MayThrow(send) = ⋃ MayThrow(t)
MayYield(send) = OR MayYield(t)
```

Must-guarantees become intersections/logical common consequences. This asymmetry is essential: may properties join broadly; guaranteed postconditions weaken to what every target promises.

## Incrementality

A caller proof may depend on:

```text
ReceiverType/flow facts
SelectorId semantics
Class hierarchy revision
Method table revision
Protocol/base contract revision
Target summaries
Reflection/world closure policy
```

If an override is added, caller body text does not change but proof validity may. The dependency graph must capture that frontier.

## Tests

- adding an override invalidates a proof based on target enumeration;
- changing only method body but not guaranteed contract does not necessarily invalidate modular callers, depending on trust/verification policy;
- changing a protocol contract invalidates callers;
- reflective method installation blocks or invalidates closed-target proof;
- `super` lookup uses correct start but current receiver;
- message-not-understood path is present when member existence is not proven;
- exact-class guard allows specialization only while dispatch revision matches;
- LSP heuristic target never enables runtime-check removal.

## Review questions

1. Is the target set a sound over-approximation?
2. What language guarantee makes it stable?
3. Are override contracts substitutable?
4. What happens if reflection changes the method table?
5. Does typing constrain the member contract or accidentally redefine dispatch?
6. What effects/throws/yields are unioned across targets?
7. Which revision invalidates the proof when class hierarchy changes?
