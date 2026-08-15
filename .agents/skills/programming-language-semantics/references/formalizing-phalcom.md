# Formalizing Phalcom: A Practical Strategy

Phalcom does not need a fully mechanized semantics before implementation can benefit from formal methods. The strongest path is to formalize a small semantic kernel, use it to resolve ambiguous decisions, and expand where formalization buys correctness.

## 1. Objectives

Formalization should answer concrete questions:

- Is evaluation order unambiguous?
- Does `super` preserve receiver correctly?
- Can block non-local return be modeled without contradictions?
- Which module-cycle states exist?
- What does checker guarantee relative to runtime dispatch?
- Which compiler lowerings preserve behavior?
- What facts survive fiber suspension?

Avoid formalizing syntax trivia with little semantic leverage.

## 2. Core calculus

A useful first Phalcom kernel:

```text
values/literals
lexical variables
mutable assignment
block creation/invocation
objects/classes
message send
inheritance + super send
field read/write
return + non-local return
throw/handler
```

Add modules/fibers after sequential object/control core stabilizes.

## 3. Abstract syntax for semantics

Conceptual grammar:

```text
e ::= v
    | x
    | x := e
    | block(params, e*)
    | send(e, selector, args)
    | superSend(selector, args)
    | fieldRead(name)
    | fieldWrite(name,e)
    | return(e)
    | nonlocalReturn(e)
    | throw(e)
    | sequence(e*)
```

Do not reproduce parser grammar. Formal syntax should expose semantic distinctions, not token details.

## 4. Domains and invariants

Define:

```text
BindingId, Loc, Value, ObjectId, ClassId, MethodId, Selector
Env, Store, Heap, FrameId, Continuation, Outcome
```

State invariants:

- `classOf(v)` total for surface values;
- class objects have metaclasses;
- superclass relation satisfies class rules;
- binding IDs identify declarations, not spellings;
- live object refs resolve in heap;
- live frame belongs to one fiber;
- selector canonicalization deterministic for same call shape.

## 5. Formalization progression

### Phase A — big-step with explicit outcomes

Useful for literals, names, assignment, sequencing, block construction, ordinary sends.

```text
ρ; σ; H ⊢ e ⇓ o; σ'; H'
```

### Phase B — abstract machine

Introduce continuation/frame state for evaluation order, exceptions, non-local return, and VM correspondence.

### Phase C — labeled small-step

Add external events and scheduler transitions for IO/modules/fibers.

Do not force one monolithic formalism from day one.

## 6. Formalize dispatch early

Dispatch is semantic center. Define reusable operations:

```text
canonicalSelector(callSyntax)
classOf(value)
lookup(startClass, selector, accessContext)
invoke(method, receiver, args)
messageMiss(...)
```

Ordinary, `super`, class-side, `perform`, and cached paths should reduce to same semantic core with different explicit inputs.

## 7. Formalize access separately

```text
permitted(accessContext, member) -> Bool
```

Call from all dispatch routes. This prevents proving direct send while reflective send diverges.

## 8. Blocks and home frames

Block values carry only semantic state needed:

```text
code + captured environment + homeFrame/homeFiber metadata
```

Establish:

- construction does not execute body;
- free variables use captured lexical storage;
- non-local return targets home frame;
- dead target produces defined error.

## 9. Core object heap

A minimal heap model:

```text
H : ObjectId -> (ClassId, FieldMap)
```

Allocation produces fresh `ObjectId`. Field update changes heap. Immediate primitives can be separate `Value` cases while still satisfying `classOf`.

## 10. Method tables and reflection state

If reflective mutation exists, class state includes versioned method table conceptually:

```text
Methods(C, epoch)
```

This allows cache/analysis theorems to state assumption "method state unchanged." If reflection is excluded from core calculus initially, record it explicitly rather than pretending immutable forever.

## 11. Module extension

Add:

```text
ModuleState = Unloaded | Loading | Initialized | Failed
```

plus namespace/module identities. Model cycles explicitly or ban them statically.

## 12. Fiber extension

Lift machine into scheduler:

```text
GlobalState = sharedHeap × modules × map(FiberId -> MachineState) × Scheduler
```

Transitions either step current fiber or schedule at allowed boundaries.

## 13. Static semantics layer

Define name resolution:

```text
Γ ⊢ occurrence ↦ target
```

Future typing:

```text
Γ ⊢ e ⇒ T
Γ ⊢ e ⇐ T
```

Then define semantic interpretation theorem connecting types to runtime values/outcomes.

## 14. Analyzer abstraction relation

For LSP/static analysis:

```text
concrete state ∈ γ(abstract state)
```

A sound transfer overapproximates all dynamic successors. This is where PL semantics hands off to abstract interpretation.

Do not prove analyzer facts by treating analyzer algorithm as concrete semantics.

## 15. Compiler relation

Define representation relation `R` between source machine and bytecode VM. Start with selected features:

- local load/store;
- allocation/field access;
- ordinary send;
- super send;
- block construction/invocation;
- return/throw.

Property-based/differential testing can stand in for full proof initially.

## 16. Mechanization options

Potential tools when warranted:

- Rocq/Coq for inductive relations/compiler proofs;
- Lean for semantics/metatheory;
- Agda for typed developments;
- Redex/K-style executable semantics for exploration;
- custom reference interpreter for engineering validation.

Tool choice follows proof/exploration goal, not prestige.

## 17. Specification artifact triad

Keep aligned:

```text
normative prose rule
compact formal/pseudocode rule
executable conformance fixture
```

A reader should move between them.

## 18. Semantic decision record

When formalization exposes ambiguity, record:

```text
Question
Competing semantics
Observable distinguishing program
Chosen rule
Consequences for compiler/LSP/checker/prover
```

This is more valuable than notation without decision.

## 19. Suggested milestones

```text
F0 domains/outcomes/observations
F1 sequential lexical state + blocks
F2 objects/metaclasses/dispatch/super/dNU
F3 abrupt control + handlers
F4 reflection/open-world assumptions
F5 modules + cycles
F6 fibers + scheduling
F7 static correspondence
F8 compiler refinement
```

## 20. What not to formalize prematurely

- exact opcode encodings;
- Rust ownership details;
- every stdlib API;
- formatter syntax;
- editor recovery nodes as executable constructs;
- speculative features without semantic decisions.

## 21. Validation against implementation

For each formal rule identify current code paths. If behavior differs classify:

```text
spec bug
implementation bug
formalization bug
known deferred feature
```

Do not silently mutate formal rule to match accidental implementation.

## 22. Competency checks

1. Why should formal core grammar differ from parser grammar?
2. Which Phalcom feature should be formalized earliest and why?
3. Where does abstract interpretation connect to concrete semantics?
4. What three artifacts should accompany important semantic rule?
5. When is proof assistant worth introducing?
6. How should reflective mutation appear if first core calculus omits it?
