# Semantic Domains and Machine State

Before writing evaluation rules, define the kinds of entities the rules manipulate. A good domain model prevents semantics from inheriting accidental Rust or bytecode representation choices.

## 1. Core domains

```text
Name        source spelling
BindingId   lexical declaration identity
ModuleId    module identity
ClassId     class identity
Selector    canonical message identity
MethodId    installed method identity
FieldId     owner + side + field identity
Loc         abstract mutable storage location
Value       runtime surface value
Env         BindingId -> Denotation
Store       Loc -> Value
Heap        object identity -> object state
FrameId     activation identity
FiberId     fiber identity
Event       externally observable action
```

These are semantic sets. They do not require the implementation to allocate a Rust struct for each one.

## 2. Values versus locations

For immutable bindings, an environment may map directly to a value:

```text
ρ : BindingId -> Value
```

For mutable bindings and captured mutation, use locations:

```text
ρ : BindingId -> Loc
σ : Loc -> Value
```

Then assignment changes the store rather than lexical identity:

```text
ρ(x) = ℓ
σ' = σ[ℓ ↦ v]
```

This cleanly explains why two closures capturing the same mutable binding observe each other's writes.

## 3. Object identity and object state

A heap object conceptually has identity independent of current fields:

```text
ObjectRef = identity token
Heap(ObjectRef) = {
    class: ClassId,
    slots: FieldId -> Value,
    ...
}
```

Object equality may be value-based or identity-based depending on operator semantics, but reflective identity constrains optimizations. If allocation can become observable through identity, it cannot be removed solely because its ordinary result is unused.

Immediate values such as small integers can bypass heap allocation while still having a total surface class. Semantic identity need not mirror representation identity.

## 4. Classes and metaclasses

Represent semantic relationships explicitly:

```text
classOf : Value -> ClassId
superclass : ClassId -> Option<ClassId>
metaclassOf : ClassId -> ClassId
methods : ClassId × Selector -> MethodId?
```

For a class object `C`:

```text
classOf(C) = metaclassOf(C)
```

This is why class-side send does not require a disconnected static namespace.

## 5. Method values versus installed methods

Separate:

- method declaration;
- runtime `Method` object exposed by reflection;
- installed entry in a method dictionary;
- bound method pairing implementation with receiver.

A semantic `MethodId` should denote behavior/installation identity relevant to reflection and invalidation without assuming bytecode pointer identity.

## 6. Frames

A method activation needs enough conceptual state for:

```text
Frame = {
    id,
    callable,
    receiver,
    locals/environment,
    callerContinuation,
    homeFiber,
    liveness
}
```

A block with non-local return may retain a `homeFrameId`. It does not need to retain the physical native stack address of that frame.

## 7. Continuations

A continuation represents what remains to be done:

```text
Kont = Halt
     | EvalArg(rest, evaluated, sendInfo, env, next)
     | ApplySend(..., next)
     | Sequence(rest, env, next)
     | ReturnTo(frame, next)
     | Handler(handler, next)
     | ...
```

This makes evaluation order and abrupt control explicit.

## 8. Modules

A module semantic record may include:

```text
ModuleState = Unloaded
            | Loading(partialNamespace)
            | Initialized(namespace)
            | Failed(error)
```

Namespace contents and initialization state are different. Decide whether cycles can observe `Loading`, whether imports bind live references or copied values, and whether failed initialization is cached/retried.

## 9. Fibers and scheduler state

```text
Scheduler = {
    current: FiberId?,
    runnable: Queue<FiberId>,
    suspended: Map<FiberId, WaitReason>,
    completed: Map<FiberId, Outcome>
}
```

A fiber contains a continuation/frame stack and fiber-local state. Cooperative scheduling means transitions between fibers occur only at specified scheduling boundaries; it does not mean scheduler semantics are unnecessary.

## 10. External world

IO, clock, process state, environment variables, filesystem, networking, randomness, and native libraries are not ordinary in-language store. Model them as an abstract world `ω` or events:

```text
⟨e, σ, ω⟩ -> ⟨e', σ', ω'⟩
```

or:

```text
C --read(path, result)--> C'
```

This keeps deterministic core evaluation separable from environmental nondeterminism.

## 11. Access context

Access control often depends on more than receiver/class:

```text
AccessContext = {
    lexicalClass,
    module,
    privilegedCore,
    reflectiveAuthority
}
```

Private/protected semantics should be expressed in terms of authority, not inferred from call-stack accidents.

## 12. Source metadata

Decide which metadata is semantically observable:

- method source location;
- documentation;
- annotations;
- lexical owner;
- declaration order;
- parameter labels/names.

If reflection exposes a property, an optimizer or lowering that destroys it must reconstruct equivalent metadata or be restricted.

## 13. State factoring principle

Do not put everything in one `State` and then treat every operation as touching everything. Factor state so effect summaries can say:

```text
writes: local ℓ1
writes: receiver field f
reads: module M binding x
mayYield: false
```

This later enables sound static reasoning and optimization.

## 14. Phalcom representation bridge

Current implementation concepts such as `ObjRef`, arena handles, stack frames, bytecode chunks, `ModuleId`, and semantic LSP IDs implement pieces of this model. Map them to semantic domains explicitly when proving or testing correspondence.

## 15. Competency checks

1. Why is `BindingId -> Value` insufficient for a mutable variable captured by two blocks?
2. Why can a small integer have a class without heap object identity?
3. What semantic state is necessary to define a dead non-local return?
4. Why should module loading state not be merged with namespace contents?
5. Which source metadata constrains transformations only if reflection exposes it?
